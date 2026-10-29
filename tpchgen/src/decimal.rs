//! [`TPCHDecimal`] and decimal handling

use crate::fmt_bytes::{push_i64, push_two_digits};
use std::fmt;

/// Represents a decimal with a scale of 2.
///
/// For example `TPCHDecimal(1234)` represents `12.34`.
///
/// A 'decimal' column should be able to fit any values in the range
/// [-9_999_999_999.99, +9_999_999_999.99] in increments of 0.01.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TPCHDecimal(pub i64);

impl TPCHDecimal {
    /// Create a new decimal value.
    ///
    /// # Example
    /// ```
    /// use tpchgen::decimal::TPCHDecimal;
    /// let decimal = TPCHDecimal::new(1234);
    /// assert_eq!(decimal.to_string(), "12.34");
    /// ```
    pub fn new(value: i64) -> Self {
        TPCHDecimal(value)
    }

    pub const ZERO: TPCHDecimal = TPCHDecimal(0);

    /// Converts the decimal value to an f64.
    ///
    /// This is a potentially lossy conversion.
    pub const fn as_f64(&self) -> f64 {
        self.0 as f64 / 100.0
    }

    /// Returns if this decimal is negative.
    pub const fn is_negative(&self) -> bool {
        self.0.is_negative()
    }

    /// Returns the digits before the decimal point.
    pub const fn int_digits(&self) -> i64 {
        (self.0 / 100).abs()
    }

    /// Returns the digits after the decimal point.
    pub const fn decimal_digits(&self) -> i64 {
        (self.0 % 100).abs()
    }

    /// Return the inner i64 value.
    pub const fn into_inner(self) -> i64 {
        self.0
    }
}

impl TPCHDecimal {
    /// Appends this decimal to `buf`, as the [`Display`](fmt::Display) impl
    /// formats it.
    #[inline]
    pub(crate) fn write_to(&self, buf: &mut Vec<u8>) {
        if self.is_negative() {
            buf.push(b'-');
        }
        push_i64(buf, self.int_digits());
        buf.push(b'.');
        push_two_digits(buf, self.decimal_digits() as u64);
    }
}

impl fmt::Display for TPCHDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}.{:0>2}",
            if self.is_negative() { "-" } else { "" },
            self.int_digits(),
            self.decimal_digits()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_format() {
        struct TestCase {
            decimal: TPCHDecimal,
            expected: &'static str,
        }

        let test_cases = [
            TestCase {
                decimal: TPCHDecimal(0),
                expected: "0.00",
            },
            TestCase {
                decimal: TPCHDecimal(1),
                expected: "0.01",
            },
            TestCase {
                decimal: TPCHDecimal(10),
                expected: "0.10",
            },
            TestCase {
                decimal: TPCHDecimal(100),
                expected: "1.00",
            },
            TestCase {
                decimal: TPCHDecimal(1000),
                expected: "10.00",
            },
            TestCase {
                decimal: TPCHDecimal(1234),
                expected: "12.34",
            },
            TestCase {
                decimal: TPCHDecimal(-1),
                expected: "-0.01",
            },
            TestCase {
                decimal: TPCHDecimal(-10),
                expected: "-0.10",
            },
            TestCase {
                decimal: TPCHDecimal(-100),
                expected: "-1.00",
            },
            TestCase {
                decimal: TPCHDecimal(-1000),
                expected: "-10.00",
            },
            // Max according to spec.
            TestCase {
                decimal: TPCHDecimal(999_999_999_999),
                expected: "9999999999.99",
            },
            // Min according to spec.
            TestCase {
                decimal: TPCHDecimal(-999_999_999_999),
                expected: "-9999999999.99",
            },
        ];

        for test_case in test_cases {
            let formatted = test_case.decimal.to_string();
            assert_eq!(
                test_case.expected, formatted,
                "input decimal: {:?}",
                test_case.decimal,
            );

            let mut buf = Vec::new();
            test_case.decimal.write_to(&mut buf);
            assert_eq!(
                test_case.expected.as_bytes(),
                buf,
                "input decimal: {:?}",
                test_case.decimal,
            );
        }
    }
}
