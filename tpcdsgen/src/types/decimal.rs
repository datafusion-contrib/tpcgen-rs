use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Decimal {
    // XXX: Definitions of precision and scale are reversed. This was done to
    // make it easier to follow the C code, which reverses the definitions. Here,
    // precision means the number of decimal places and scale means the total number
    // of digits. We leave out the scale field because it's never used, and the C implementation
    // was buggy.
    precision: i32,
    number: i64,
}

impl Decimal {
    pub const ZERO: Decimal = Decimal {
        number: 0,
        precision: 2,
    };
    pub const ONE_HALF: Decimal = Decimal {
        number: 50,
        precision: 2,
    };
    pub const NINE_PERCENT: Decimal = Decimal {
        number: 9,
        precision: 2,
    };
    pub const ONE_HUNDRED: Decimal = Decimal {
        number: 10000,
        precision: 2,
    };
    pub const ONE: Decimal = Decimal {
        number: 100,
        precision: 2,
    };

    /// Panics if `precision` is negative.
    pub fn new(number: i64, precision: i32) -> Self {
        assert!(
            precision >= 0,
            "precision must be greater than or equal to zero"
        );
        Decimal { precision, number }
    }

    pub fn parse_decimal(decimal_string: &str) -> Result<Self> {
        let number: i64;
        let precision: i32;

        if let Some(decimal_point_index) = decimal_string.find('.') {
            let fractional = &decimal_string[decimal_point_index + 1..];
            precision = fractional.len() as i32;
            let integer_part = &decimal_string[..decimal_point_index];
            let combined = format!("{}{}", integer_part, fractional);
            number = combined
                .parse::<i64>()
                .map_err(|_| crate::TpcdsError::new("Failed to parse decimal string"))?;
        } else {
            number = decimal_string
                .parse::<i64>()
                .map_err(|_| crate::TpcdsError::new("Failed to parse decimal string"))?;
            precision = 0;
        }

        Ok(Self::new(number, precision))
    }

    pub fn add2(decimal1: Decimal, decimal2: Decimal) -> Decimal {
        let precision = if decimal1.precision > decimal2.precision {
            decimal1.precision
        } else {
            decimal2.precision
        };
        // This is not mathematically correct when the precisions aren't the same, but it's what the C code does
        let number = decimal1.number + decimal2.number;
        Decimal { number, precision }
    }

    pub fn subtract(decimal1: Decimal, decimal2: Decimal) -> Decimal {
        let precision = if decimal1.precision > decimal2.precision {
            decimal1.precision
        } else {
            decimal2.precision
        };
        // again following C code
        let number = decimal1.number - decimal2.number;
        Decimal { number, precision }
    }

    pub fn multiply(decimal1: Decimal, decimal2: Decimal) -> Decimal {
        let precision = if decimal1.precision > decimal2.precision {
            decimal1.precision
        } else {
            decimal2.precision
        };
        let mut number = decimal1.number * decimal2.number;
        for _i in (precision + 1)..=(decimal1.precision + decimal2.precision) {
            number /= 10; // Always round down, I guess
        }
        Decimal { number, precision }
    }

    pub fn divide(decimal1: Decimal, decimal2: Decimal) -> Decimal {
        let mut f1 = decimal1.number as f32;
        let precision = if decimal1.precision > decimal2.precision {
            decimal1.precision
        } else {
            decimal2.precision
        };

        for _i in decimal1.precision..precision {
            f1 *= 10.0;
        }

        for _i in 0..precision {
            f1 *= 10.0;
        }

        let mut f2 = decimal2.number as f32;
        for _i in decimal2.precision..precision {
            f2 *= 10.0;
        }

        let number = (f1 / f2) as i64;
        Decimal { number, precision }
    }

    pub fn negate(decimal: Decimal) -> Decimal {
        Decimal {
            number: -decimal.number,
            precision: decimal.precision,
        }
    }

    pub fn from_integer(from: i32) -> Decimal {
        Decimal {
            number: from as i64,
            precision: 0,
        }
    }

    pub fn get_precision(&self) -> i32 {
        self.precision
    }

    pub fn get_number(&self) -> i64 {
        self.number
    }
}

impl std::fmt::Display for Decimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The C code (print_decimal in print.c) converts to double and prints
        // with %.*f. For the magnitudes TPC-DS produces (far below 2^53) the
        // double round-trip always recovers the exact digits of `number`, so
        // we format the integer directly and avoid float formatting entirely.
        if self.precision <= 0 {
            return write!(f, "{}", self.number);
        }
        let divisor = 10i64.pow(self.precision as u32);
        let int_part = self.number / divisor;
        let frac = (self.number % divisor).unsigned_abs();
        let width = self.precision as usize;
        if self.number < 0 && int_part == 0 {
            // integer division dropped the sign (e.g. -0.05)
            write!(f, "-0.{:0width$}", frac)
        } else {
            write!(f, "{}.{:0width$}", int_part, frac)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_creation() {
        let decimal = Decimal::new(12345, 2);
        assert_eq!(decimal.get_number(), 12345);
        assert_eq!(decimal.get_precision(), 2);
    }

    #[test]
    fn test_parse_decimal() {
        let decimal = Decimal::parse_decimal("123.45").unwrap();
        assert_eq!(decimal.get_number(), 12345);
        assert_eq!(decimal.get_precision(), 2);

        let decimal = Decimal::parse_decimal("123").unwrap();
        assert_eq!(decimal.get_number(), 123);
        assert_eq!(decimal.get_precision(), 0);
    }

    #[test]
    fn test_constants() {
        assert_eq!(Decimal::ZERO.get_number(), 0);
        assert_eq!(Decimal::ONE.get_number(), 100);
        assert_eq!(Decimal::ONE_HUNDRED.get_number(), 10000);
    }

    #[test]
    fn test_arithmetic() {
        let d1 = Decimal::new(100, 2); // 1.00
        let d2 = Decimal::new(50, 2); // 0.50

        let sum = Decimal::add2(d1, d2);
        assert_eq!(sum.get_number(), 150); // Buggy behavior: should be 150, not mathematically correct

        let diff = Decimal::subtract(d1, d2);
        assert_eq!(diff.get_number(), 50);
    }

    #[test]
    fn test_display() {
        let decimal = Decimal::new(12345, 2);
        assert_eq!(format!("{}", decimal), "123.45");

        let decimal = Decimal::new(123, 0);
        assert_eq!(format!("{}", decimal), "123");
    }

    #[test]
    fn test_display_negative() {
        let decimal = Decimal::new(-12345, 2);
        assert_eq!(format!("{}", decimal), "-123.45");

        // sign must survive a zero integer part
        let decimal = Decimal::new(-5, 2);
        assert_eq!(format!("{}", decimal), "-0.05");
    }

    #[test]
    fn test_display_zero_padding() {
        let decimal = Decimal::new(5, 2);
        assert_eq!(format!("{}", decimal), "0.05");

        let decimal = Decimal::new(10005, 4);
        assert_eq!(format!("{}", decimal), "1.0005");

        let decimal = Decimal::new(0, 2);
        assert_eq!(format!("{}", decimal), "0.00");
    }

    #[test]
    #[should_panic(expected = "precision must be greater than or equal to zero")]
    fn test_negative_precision_panics() {
        Decimal::new(1, -1);
    }
}
