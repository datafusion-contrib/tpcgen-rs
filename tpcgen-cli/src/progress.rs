//! Progress tracking for data generation.
//!
//! # Overview
//!
//! [`ProgressTracker`] is a small, dyn-compatible trait that owns run-level
//! progress lifecycle. Generation code uses it as follows:
//!
//! 1. [`ProgressTracker::register`] once per progress item, before work
//!    starts, with the total number of output units the item will produce
//!    (chunks or rows for text output, row groups for Parquet). Registration
//!    returns a [`ProgressHandle`] bound to that item.
//! 2. [`ProgressTracker::start`] once after all known progress items have
//!    been registered and before work starts. This hook is optional for
//!    paths that register items lazily.
//! 3. [`ProgressHandle::increment`] after output units are written.
//!    Cloned handles may be advanced concurrently by generation tasks.
//! 4. [`ProgressHandle::complete`] after an item's output is committed.
//!    This is optional for paths without a distinct item-completion boundary.
//! 5. [`ProgressTracker::finish`] after the generation run completes
//!    successfully.
//!
//! Registration and run-level lifecycle callbacks are invoked serially.
//!
//! Implementations must not panic and must not propagate I/O errors —
//! progress reporting is best-effort and must never affect the data
//! path.
//!
//! # Default implementation
//!
//! When the `indicatif-progress` feature is enabled (on by default), the
//! crate provides an `IndicatifProgress` implementation, which renders
//! one progress bar per progress item on stderr using the `indicatif` crate.
//! Library users who do not want to pull in `indicatif` can disable default
//! features and still supply their own [`ProgressTracker`] implementation.
//! Without `indicatif-progress` and without a custom tracker, progress
//! reporting is a no-op.
//!
//! # Example: a custom logging tracker
//!
//! ```
//! use std::sync::atomic::{AtomicU64, Ordering};
//! use std::sync::Arc;
//! use tpcgen_cli::progress::{ProgressHandle, ProgressTracker};
//!
//! #[derive(Debug)]
//! struct LoggingTracker {
//!     written: AtomicU64,
//! }
//!
//! impl ProgressTracker for LoggingTracker {
//!     fn register(self: Arc<Self>, item: &str, total: u64) -> ProgressHandle {
//!         eprintln!("plan: {item} -> {total} output units");
//!         ProgressHandle::new(move |units| {
//!             self.written.fetch_add(units, Ordering::Relaxed);
//!         })
//!     }
//!     fn finish(&self) {
//!         eprintln!("done: {} output units", self.written.load(Ordering::Relaxed));
//!     }
//! }
//! ```

use std::fmt;
use std::sync::Arc;

/// Receives generation-progress events for one generation run.
///
/// See the [module-level documentation](self) for the call-order
/// contract. Trackers are held in a [`std::sync::Arc`] and shared across
/// concurrent generation tasks through the [`ProgressHandle`]s returned by
/// [`Self::register`], so they must be `Send + Sync`.
/// They must also be `Debug` so containing types can derive `Debug`.
pub trait ProgressTracker: Send + Sync + fmt::Debug {
    /// Pre-register a progress item with its total expected output-unit count.
    ///
    /// Called once per item before work starts. The `item` is a stable
    /// identifier, usually a table name. The returned handle is the only
    /// capability generation tasks need to report item progress and optional
    /// completion.
    ///
    /// The [`Arc`] receiver lets implementations return a `'static` handle
    /// that shares tracker state with concurrent generation tasks.
    fn register(self: Arc<Self>, item: &str, total_units: u64) -> ProgressHandle;

    /// Optional hook called after all known progress items have been
    /// registered and before the first [`ProgressHandle::increment`].
    ///
    /// Implementations can use this to finalize setup that depends on the
    /// registered item set. The default does nothing.
    fn start(&self) {}

    /// Called once when the generation run completes successfully.
    /// The default does nothing.
    fn finish(&self) {}
}

/// A cloneable handle for reporting progress and optional completion for one
/// registered item.
///
/// Handles are created by [`ProgressTracker::register`] to report item-specific
/// progress and optional completion. Run-level lifecycle operations remain the
/// responsibility of the tracker owner.
#[derive(Clone)]
pub struct ProgressHandle {
    increment: Arc<dyn Fn(u64) + Send + Sync>,
    complete: Arc<dyn Fn() + Send + Sync>,
}

impl ProgressHandle {
    /// Create a handle for reporting item progress with no completion callback.
    pub fn new<F>(increment: F) -> Self
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        Self::new_with_complete(increment, || {})
    }

    /// Create a handle for reporting item progress and completion.
    pub fn new_with_complete<F, C>(increment: F, complete: C) -> Self
    where
        F: Fn(u64) + Send + Sync + 'static,
        C: Fn() + Send + Sync + 'static,
    {
        Self {
            increment: Arc::new(increment),
            complete: Arc::new(complete),
        }
    }

    /// Advance this item's counter by `units` output units.
    pub fn increment(&self, units: u64) {
        (self.increment)(units);
    }

    /// Optionally notify the tracker that this item completed successfully.
    ///
    /// Generation paths that cannot identify an independent item-completion
    /// boundary may rely on [`ProgressTracker::finish`] instead.
    pub fn complete(&self) {
        (self.complete)();
    }

    fn no_op() -> Self {
        Self::new(|_| {})
    }
}

impl fmt::Debug for ProgressHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProgressHandle").finish_non_exhaustive()
    }
}

/// Default tracker used when no progress backend is installed.
///
/// This keeps generation on the same always-reporting path; the no-op
/// implementation simply ignores every event.
#[derive(Debug, Default)]
struct NoOpProgressTracker;

impl ProgressTracker for NoOpProgressTracker {
    fn register(self: Arc<Self>, _item: &str, _total_units: u64) -> ProgressHandle {
        ProgressHandle::no_op()
    }
}

pub(crate) fn no_op_progress_tracker() -> Arc<dyn ProgressTracker> {
    Arc::new(NoOpProgressTracker)
}

#[cfg(feature = "indicatif-progress")]
pub use indicatif_impl::IndicatifProgress;

#[cfg(feature = "indicatif-progress")]
mod indicatif_impl {
    use super::{ProgressHandle, ProgressTracker};
    #[cfg(test)]
    use indicatif::ProgressDrawTarget;
    use indicatif::{
        HumanDuration, MultiProgress, ProgressBar, ProgressFinish, ProgressState, ProgressStyle,
    };
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
    use std::time::{Duration, Instant};

    const LABEL_WIDTH: usize = 22;
    const BAR_WIDTH: usize = 18;
    const PROGRESS_FLUSH_INTERVAL: Duration = Duration::from_millis(200);
    const PROGRESS_CHARS: &str = "=>-";

    /// Default [`ProgressTracker`] implementation backed by
    /// [`indicatif::MultiProgress`].
    ///
    /// Renders one compact progress bar per progress item on stderr.
    ///
    /// Items are added in [`ProgressTracker::register`], which returns a handle
    /// that advances and completes its progress bar directly.
    #[derive(Debug)]
    pub struct IndicatifProgress {
        multi: MultiProgress,
        bars: Mutex<Vec<ProgressBar>>,
    }

    impl IndicatifProgress {
        /// Construct an empty tracker. Progress items are added via
        /// [`ProgressTracker::register`].
        pub fn new() -> Self {
            Self {
                multi: MultiProgress::new(),
                bars: Mutex::new(Vec::new()),
            }
        }

        /// Return a writer that coordinates stderr log writes with progress
        /// bar redraws.
        pub fn log_writer(&self) -> Box<dyn io::Write + Send + 'static> {
            Box::new(IndicatifLogWriter {
                multi: self.multi.clone(),
            })
        }

        fn lock_bars(&self) -> MutexGuard<'_, Vec<ProgressBar>> {
            self.bars
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
        }

        #[cfg(test)]
        fn hidden() -> Self {
            Self {
                multi: MultiProgress::with_draw_target(ProgressDrawTarget::hidden()),
                bars: Mutex::new(Vec::new()),
            }
        }
    }

    impl Default for IndicatifProgress {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ProgressTracker for IndicatifProgress {
        fn register(self: Arc<Self>, item: &str, total_units: u64) -> ProgressHandle {
            // Indicatif treats zero-length items as complete.
            let total = total_units.max(1);
            let bar = self.multi.add(
                ProgressBar::new(total)
                    .with_style(bar_style())
                    .with_message(item.to_owned())
                    .with_finish(ProgressFinish::AndLeave),
            );
            self.lock_bars().push(bar.clone());
            throttled_progress_handle(bar, total)
        }

        fn start(&self) {
            let bars = self.lock_bars().clone();

            // Populate every bar's initial draw state, then force one render so
            // the full registered table set is visible before generation starts.
            for bar in &bars {
                bar.tick();
            }
            if let Some(bar) = bars.last() {
                bar.force_draw();
            }
        }

        fn finish(&self) {
            let bars = self.lock_bars().clone();

            for bar in bars {
                bar.finish_using_style();
            }
        }
    }

    // ProgressBar::inc enters indicatif's render path. When many threads report
    // row-level progress, calling it for every row can repeatedly redraw the
    // cursor, so batch deltas before forwarding them to indicatif.
    fn throttled_progress_handle(bar: ProgressBar, total: u64) -> ProgressHandle {
        throttled_progress_handle_with_clock(bar, total, Instant::now)
    }

    fn throttled_progress_handle_with_clock<N>(
        bar: ProgressBar,
        total: u64,
        now: N,
    ) -> ProgressHandle
    where
        N: Fn() -> Instant + Send + Sync + 'static,
    {
        let progress = Arc::new(ThrottledProgress::new(bar, total, now));
        let increment = progress.clone();
        let complete = progress;

        ProgressHandle::new_with_complete(
            move |units| increment.increment(units),
            move || complete.complete(),
        )
    }

    struct ThrottledProgress {
        bar: ProgressBar,
        total: u64,
        throttle: Mutex<ThrottleState>,
        clock: Arc<dyn Fn() -> Instant + Send + Sync>,
    }

    impl ThrottledProgress {
        fn new<N>(bar: ProgressBar, total: u64, now: N) -> Self
        where
            N: Fn() -> Instant + Send + Sync + 'static,
        {
            let next_flush = now();
            Self {
                bar,
                total,
                throttle: Mutex::new(ThrottleState {
                    pending: 0,
                    next_flush,
                }),
                clock: Arc::new(now),
            }
        }

        fn increment(&self, units: u64) {
            if units == 0 {
                return;
            }

            let mut throttle = self
                .throttle
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if self.bar.is_finished() {
                throttle.clear();
                return;
            }

            let position = self.bar.position();
            if position == 0 {
                // Reset before the first increment so queued time is excluded from the ETA.
                self.bar.reset_eta();
            }

            throttle.pending = throttle.pending.saturating_add(units);
            let now = (self.clock)();
            if now < throttle.next_flush {
                return;
            }

            throttle.next_flush = now + PROGRESS_FLUSH_INTERVAL;
            let remaining_to_total = self.total.saturating_sub(position);
            let flush_units = std::mem::take(&mut throttle.pending).min(remaining_to_total);
            if flush_units == 0 {
                return;
            }
            self.bar.inc(flush_units);
            if self.bar.position() >= self.total {
                // Finish exact-count items even when the caller does not report completion.
                self.bar.finish_using_style();
            }
        }

        fn complete(&self) {
            let mut throttle = self
                .throttle
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            throttle.clear();
            if self.bar.is_finished() {
                return;
            }

            self.bar.finish_using_style();
        }
    }

    struct ThrottleState {
        pending: u64,
        next_flush: Instant,
    }

    impl ThrottleState {
        fn clear(&mut self) {
            self.pending = 0;
        }
    }

    fn bar_style() -> ProgressStyle {
        static STYLE: OnceLock<ProgressStyle> = OnceLock::new();
        STYLE
            .get_or_init(|| {
                ProgressStyle::default_bar()
                    .template(&bar_template())
                    .expect("progress bar template is valid")
                    .progress_chars(PROGRESS_CHARS)
                    .with_key("status", write_progress_status)
            })
            .clone()
    }

    fn bar_template() -> String {
        format!(
            "{{msg:!{LABEL_WIDTH}}} [{{bar:{BAR_WIDTH}.cyan/blue}}] ({{percent:>3}}%){{status}}"
        )
    }

    fn write_progress_status(state: &ProgressState, writer: &mut dyn std::fmt::Write) {
        let started = state.pos() > 0;
        if state.is_finished() {
            let _ = write!(writer, " done");
        } else if !started {
            let _ = write!(writer, " ETA --");
        } else {
            let _ = write!(writer, " ETA {:#}", HumanDuration(state.eta()));
        }
    }

    struct IndicatifLogWriter {
        multi: MultiProgress,
    }

    impl Write for IndicatifLogWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.multi.suspend(|| {
                let mut stderr = io::stderr().lock();
                stderr.write(buf)
            })
        }

        fn flush(&mut self) -> io::Result<()> {
            self.multi.suspend(|| {
                let mut stderr = io::stderr().lock();
                stderr.flush()
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn registers_and_increments() {
            let t = Arc::new(IndicatifProgress::hidden());
            let progress = [
                t.clone().register("lineitem", 60),
                t.clone().register("orders", 15),
            ];
            progress[0].increment(1);
            progress[1].increment(5);

            let bars = t.bars.lock().unwrap();
            assert_eq!(bars[0].position(), 1);
            assert_eq!(bars[1].position(), 5);
        }

        #[test]
        fn zero_total_items_start_at_zero() {
            let t = Arc::new(IndicatifProgress::hidden());
            t.clone().register("store_returns", 0);

            let bars = t.bars.lock().unwrap();
            assert_eq!(bars[0].position(), 0);
            assert_eq!(bars[0].length(), Some(1));
            assert!(!bars[0].is_finished());
        }

        #[test]
        fn reaching_total_finishes_item() {
            let t = Arc::new(IndicatifProgress::hidden());
            let progress = t.clone().register("orders", 5);
            progress.increment(5);

            let bars = t.bars.lock().unwrap();
            assert_eq!(bars[0].position(), 5);
            assert!(bars[0].is_finished());
        }

        #[test]
        fn large_items_coalesce_small_increments() {
            let start = Instant::now();
            let now = Arc::new(Mutex::new(start));
            let clock = now.clone();
            let bar = ProgressBar::with_draw_target(Some(10_000), ProgressDrawTarget::hidden());
            let progress = throttled_progress_handle_with_clock(bar.clone(), 10_000, move || {
                *clock.lock().unwrap()
            });

            progress.increment(1);
            assert_eq!(bar.position(), 1);

            for _ in 1..50 {
                progress.increment(1);
            }

            assert_eq!(bar.position(), 1);

            *now.lock().unwrap() = start + PROGRESS_FLUSH_INTERVAL;
            progress.increment(1);

            assert_eq!(bar.position(), 51);
        }

        #[test]
        fn extra_increments_after_finish_do_not_advance_item() {
            let t = Arc::new(IndicatifProgress::hidden());
            let progress = t.clone().register("store_returns", 1);

            progress.increment(1);
            progress.increment(10);

            let bars = t.bars.lock().unwrap();
            assert_eq!(bars[0].position(), 1);
            assert!(bars[0].is_finished());
        }

        #[test]
        fn completion_finishes_after_pending_increments() {
            let start = Instant::now();
            let clock = Arc::new(Mutex::new(start));
            let bar = ProgressBar::with_draw_target(Some(5), ProgressDrawTarget::hidden());
            let progress = throttled_progress_handle_with_clock(bar.clone(), 5, move || {
                *clock.lock().unwrap()
            });

            progress.increment(2);
            assert_eq!(bar.position(), 2);

            progress.increment(1);
            progress.increment(1);
            assert_eq!(bar.position(), 2);

            progress.complete();

            assert_eq!(bar.position(), 5);
            assert!(bar.is_finished());
        }

        #[test]
        fn reaching_one_total_does_not_finish_other_items() {
            let t = Arc::new(IndicatifProgress::hidden());
            let orders = t.clone().register("orders", 5);
            let _lineitem = t.clone().register("lineitem", 10);
            orders.increment(5);

            let bars = t.bars.lock().unwrap();
            assert_eq!(bars[0].position(), 5);
            assert!(bars[0].is_finished());
            assert!(!bars[1].is_finished());
        }

        #[test]
        fn explicit_completion_finishes_item_below_total() {
            let t = Arc::new(IndicatifProgress::hidden());
            let progress = t.clone().register("catalog_returns", 10);
            progress.increment(9);

            progress.complete();

            let bars = t.bars.lock().unwrap();
            assert_eq!(bars[0].position(), 10);
            assert!(bars[0].is_finished());
        }

        #[test]
        fn finish_marks_registered_items_finished() {
            let t = Arc::new(IndicatifProgress::hidden());
            let progress = t.clone().register("orders", 2);
            progress.increment(1);
            assert!(!t.bars.lock().unwrap()[0].is_finished());

            t.finish();

            assert!(t.bars.lock().unwrap()[0].is_finished());
        }

        #[test]
        fn progress_status_reflects_lifecycle() {
            let bar = ProgressBar::with_draw_target(Some(10), ProgressDrawTarget::hidden());

            assert_eq!(rendered_status(&bar), " ETA --");

            bar.inc(1);
            let active_status = rendered_status(&bar);
            assert!(active_status.starts_with(" ETA "));
            assert_ne!(active_status, " ETA --");

            bar.finish();
            assert_eq!(rendered_status(&bar), " done");
        }

        fn rendered_status(bar: &ProgressBar) -> String {
            let mut status = String::new();
            bar.update(|state| write_progress_status(state, &mut status));
            status
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    };

    /// Mock implementation that records every event. Demonstrates that
    /// the trait is dyn-compatible and usable from external code without
    /// pulling in `indicatif`.
    #[derive(Debug, Default)]
    struct MockTracker {
        registered: Mutex<Vec<(String, u64)>>,
        increments: Mutex<Vec<(String, u64)>>,
        finished: AtomicU64,
    }

    impl ProgressTracker for MockTracker {
        fn register(self: Arc<Self>, item: &str, total_units: u64) -> ProgressHandle {
            let item = item.to_owned();
            self.registered
                .lock()
                .unwrap()
                .push((item.clone(), total_units));
            ProgressHandle::new(move |units| {
                self.increments.lock().unwrap().push((item.clone(), units));
            })
        }
        fn finish(&self) {
            self.finished.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn mock_tracker_works_through_progress_handles() {
        let mock = Arc::new(MockTracker::default());
        let dynamic: Arc<dyn ProgressTracker> = mock.clone();
        let progress = [
            dynamic.clone().register("store_sales", 10),
            dynamic.clone().register("catalog_returns", 4),
        ];
        progress[0].increment(3);
        progress[1].increment(1);
        dynamic.finish();

        assert_eq!(
            *mock.registered.lock().unwrap(),
            vec![
                ("store_sales".to_owned(), 10),
                ("catalog_returns".to_owned(), 4)
            ]
        );
        assert_eq!(
            *mock.increments.lock().unwrap(),
            vec![
                ("store_sales".to_owned(), 3),
                ("catalog_returns".to_owned(), 1)
            ]
        );
        assert_eq!(mock.finished.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn progress_handle_runs_complete_callback() {
        let completed = Arc::new(AtomicU64::new(0));
        let completed_for_callback = completed.clone();

        let progress = ProgressHandle::new_with_complete(
            |_| {},
            move || {
                completed_for_callback.fetch_add(1, Ordering::Relaxed);
            },
        );

        progress.complete();

        assert_eq!(completed.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn default_start_and_finish_are_noops() {
        #[derive(Debug)]
        struct Minimal(AtomicU64);
        impl ProgressTracker for Minimal {
            fn register(self: Arc<Self>, _item: &str, _total_units: u64) -> ProgressHandle {
                ProgressHandle::new(move |units| {
                    self.0.fetch_add(units, Ordering::Relaxed);
                })
            }
        }
        let m = Arc::new(Minimal(AtomicU64::new(0)));
        let progress = m.clone().register("region", 99);
        m.start(); // no-op default
        progress.increment(7);
        progress.complete(); // no-op completion callback
        m.finish(); // no-op default
        assert_eq!(m.0.load(Ordering::Relaxed), 7);
    }
}
