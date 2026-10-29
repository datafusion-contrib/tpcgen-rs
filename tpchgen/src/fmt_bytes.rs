//! Formatting of numbers directly into byte buffers.
//!
//! These functions write the same text as the [`std::fmt`] formatting they
//! replace (`{}` and `{:0N}`), without the overhead of [`std::fmt`]. This
//! is significant when writing many rows: with [`std::fmt`], formatting is
//! most of the time spent writing TBL files.

/// The two digit strings "00" to "99", in order.
const DIGIT_PAIRS: &[u8; 200] = b"\
0001020304050607080910111213141516171819\
2021222324252627282930313233343536373839\
4041424344454647484950515253545556575859\
6061626364656667686970717273747576777879\
8081828384858687888990919293949596979899";

/// Appends the two digits of `v` (which must be less than 100) to `buf`.
#[inline]
pub(crate) fn push_two_digits(buf: &mut Vec<u8>, v: u64) {
    let d = v as usize * 2;
    buf.extend_from_slice(&DIGIT_PAIRS[d..d + 2]);
}

/// Appends the decimal digits of `v`, zero padded to at least `width`
/// digits, to `buf`.
#[inline]
fn push_u64_padded(buf: &mut Vec<u8>, mut v: u64, width: usize) {
    let mut tmp = [b'0'; 20];
    let mut i = tmp.len();
    while v >= 100 {
        let d = (v % 100) as usize * 2;
        v /= 100;
        i -= 2;
        tmp[i..i + 2].copy_from_slice(&DIGIT_PAIRS[d..d + 2]);
    }
    if v >= 10 {
        let d = v as usize * 2;
        i -= 2;
        tmp[i..i + 2].copy_from_slice(&DIGIT_PAIRS[d..d + 2]);
    } else {
        i -= 1;
        tmp[i] = b'0' + v as u8;
    }
    // `tmp` is filled with '0', so padding up to its length only moves the start
    if width > tmp.len() {
        buf.resize(buf.len() + width - tmp.len(), b'0');
    }
    let start = i.min(tmp.len().saturating_sub(width));
    buf.extend_from_slice(&tmp[start..]);
}

/// Appends `v` to `buf`, as `write!(buf, "{v}")` does.
#[inline]
pub(crate) fn push_i64(buf: &mut Vec<u8>, v: i64) {
    push_i64_padded(buf, v, 0);
}

/// Appends `v` to `buf`, zero padded to `width` characters including the
/// sign, as `write!(buf, "{v:0width$}")` does.
#[inline]
pub(crate) fn push_i64_padded(buf: &mut Vec<u8>, v: i64, width: usize) {
    if v < 0 {
        buf.push(b'-');
        push_u64_padded(buf, v.unsigned_abs(), width.saturating_sub(1));
    } else {
        push_u64_padded(buf, v as u64, width);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALUES: &[i64] = &[
        0,
        1,
        9,
        10,
        99,
        100,
        101,
        999,
        1000,
        12345,
        123456789,
        1234567890,
        -1,
        -9,
        -10,
        -99,
        -100,
        -12345,
        i64::MAX,
        i64::MIN,
        i64::MIN + 1,
    ];

    #[test]
    fn push_i64_matches_display() {
        for &v in VALUES {
            let mut buf = Vec::new();
            push_i64(&mut buf, v);
            assert_eq!(String::from_utf8(buf).unwrap(), format!("{v}"));
        }
    }

    #[test]
    fn push_i64_padded_matches_display() {
        for &v in VALUES {
            for width in [0, 1, 2, 3, 4, 9, 20, 25] {
                let mut buf = Vec::new();
                push_i64_padded(&mut buf, v, width);
                assert_eq!(
                    String::from_utf8(buf).unwrap(),
                    format!("{v:0width$}"),
                    "value {v} width {width}"
                );
            }
        }
    }

    #[test]
    fn push_two_digits_matches_display() {
        for v in 0..100 {
            let mut buf = Vec::new();
            push_two_digits(&mut buf, v);
            assert_eq!(String::from_utf8(buf).unwrap(), format!("{v:02}"));
        }
    }

    #[test]
    fn appends_to_existing_content() {
        let mut buf = b"x|".to_vec();
        push_i64_padded(&mut buf, 42, 4);
        assert_eq!(buf, b"x|0042");
    }
}
