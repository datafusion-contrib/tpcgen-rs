//! Progress registration shared by the TPC-DS outputs.
//!
//! Every output registers one bar per table, sized to the chunks it will
//! generate, and shares that bar across the table's `--parts` chunks.

use crate::progress::ProgressHandle;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Split one registered handle into `num_parts` clones that all report to the
/// same bar.
///
/// Each part finishes independently and calls [`ProgressHandle::complete`] on
/// its own, so we need to ensure the bar only reaches its "done" style once
/// every part has completed.
pub(super) fn share_handle_across_parts(
    handle: ProgressHandle,
    num_parts: usize,
) -> Vec<ProgressHandle> {
    let remaining = Arc::new(AtomicUsize::new(num_parts.max(1)));
    (0..num_parts.max(1))
        .map(|_| {
            let remaining = remaining.clone();
            let increment_handle = handle.clone();
            let complete_handle = handle.clone();
            ProgressHandle::new_with_complete(
                move |units| increment_handle.increment(units),
                move || {
                    if remaining.fetch_sub(1, Ordering::AcqRel) == 1 {
                        complete_handle.complete();
                    }
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;

    /// A handle plus counters for what it observed, for asserting on
    /// [`share_handle_across_parts`]'s forwarding behavior.
    fn recording_handle() -> (ProgressHandle, Arc<AtomicU64>, Arc<AtomicUsize>) {
        let increments = Arc::new(AtomicU64::new(0));
        let completions = Arc::new(AtomicUsize::new(0));
        let handle = {
            let increments = increments.clone();
            let completions = completions.clone();
            ProgressHandle::new_with_complete(
                move |units| {
                    increments.fetch_add(units, Ordering::Relaxed);
                },
                move || {
                    completions.fetch_add(1, Ordering::Relaxed);
                },
            )
        };
        (handle, increments, completions)
    }

    #[test]
    fn share_handle_across_parts_forwards_every_increment() {
        let (handle, increments, _completions) = recording_handle();
        let parts = share_handle_across_parts(handle, 4);
        assert_eq!(parts.len(), 4);

        for part in &parts {
            part.increment(1);
        }
        assert_eq!(increments.load(Ordering::Relaxed), 4);
    }

    #[test]
    fn share_handle_across_parts_completes_only_after_every_part_completes() {
        let (handle, _increments, completions) = recording_handle();
        let parts = share_handle_across_parts(handle, 3);

        parts[0].complete();
        parts[1].complete();
        assert_eq!(
            completions.load(Ordering::Relaxed),
            0,
            "must not finish the bar before the last part reports completion"
        );

        parts[2].complete();
        assert_eq!(completions.load(Ordering::Relaxed), 1);
    }
}
