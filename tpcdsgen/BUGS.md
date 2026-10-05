# Known Bugs and Implementation Details

This is a list of documented "bugs" and implementation details that originate in the C
implementation and were preserved in the Java port and in this Rust port as well.

It's necessary to keep the implementations bug for bug compatible for reproducibility
any deviations, even fixing some of the obvious bugs will produce different data and invalidate
benchmark results that users of this library will rely on.

### 1. Quarter Sequence Calculation Bug

**Location:** `DateDimRowGenerator.java:66`

```java
int dQuarterSeq = (dYear - 1900) * 4 + dMoy / 3 + 1;
```

**Issue:** The formula uses `dMoy / 3` instead of `(dMoy - 1) / 3`, which incorrectly assigns:
- January (month 1): `1/3 = 0` → Q1 ✓ (correct)
- February (month 2): `2/3 = 0` → Q1 ✓ (correct)
- March (month 3): `3/3 = 1` → Q2 ✗ (should be Q1)
- April (month 4): `4/3 = 1` → Q2 ✓ (correct)

This causes March to be incorrectly assigned to Q2 instead of Q1.

### 2. Weekend Days Bug

**Location:** `DateDimRowGenerator.java:76`

```java
boolean dWeekend = dDow == 5 || dDow == 6;
```

**Issue:** Marks Friday (5) and Saturday (6) as weekend days instead of Saturday (6) and Sunday (0). This is inconsistent with standard calendar conventions where weekends are Saturday and Sunday.

### 3. Current Day Comparison Bug

**Location:** `DateDimRowGenerator.java:91`

```java
boolean dCurrentDay = dDateSk == TODAYS_DATE.getDay();
```

**Issue:** Compares `dDateSk` (Julian day number, e.g., 2452663) with `TODAYS_DATE.getDay()` (day of month, e.g., 8). These are incompatible values:
- `dDateSk` is the number of days since the Julian epoch
- `getDay()` returns the day of the month (1-31)

This comparison will always be false unless by extreme coincidence the Julian day number happens to match a day of month (1-31).

### 4. Leap Year Calculation Bug

**Location:** `Date.java:100-105`

```java
public static boolean isLeapYear(int year)
{
    // This is NOT a correct computation of leap years.
    // There is a bug in the C code that doesn't handle century years correctly.
    return year % 4 == 0;
}
```

**Issue:** The implementation only checks if a year is divisible by 4, ignoring the Gregorian calendar rules:
- Years divisible by 100 are NOT leap years (e.g., 1900)
- EXCEPT years divisible by 400 ARE leap years (e.g., 2000)

The Java code itself acknowledges this is wrong but maintains it for compatibility with the C implementation.

### 5. Last Day of Month Calculation Bug

**Location:** `Date.java:computeLastDateOfMonth`

```java
public static Date computeLastDateOfMonth(Date date)
{
    // copies a bug in the C code that adds all the days in the year
    // through the first of month instead of just the number of days in the month
    return fromJulianDays(toJulianDays(date) - date.day + getDaysThroughFirstOfMonth(date));
}
```

**Issue:** The comment explicitly states this copies a bug from the C code. Instead of simply calculating the last day of the month, it performs a convoluted calculation involving days through the first of the month.

### 6. Following Holiday Bug

**Location:** `DateDimRowGenerator.java:78-82`

```java
if (dayIndex == 1) {
    // This is not correct-- the last day of the previous year is always the 366th day.
    // Copying behavior of C code.
    int lastDayOfPreviousYear = 365 + (isLeapYear(dYear - 1) ? 1 : 0);
    dFollowingHoliday = getIsHolidayFlagAtIndex(lastDayOfPreviousYear) != 0;
}
```

**Issue:** The comment explicitly states "This is not correct" but maintains the bug for C code compatibility. The issue is that the last day of the previous year should always be day 366 in the distribution array, but the code calculates it as 365 or 366 based on whether the previous year was a leap year.

## Implementation Notes

### Constants

The Java implementation uses these constants for date calculations:

```java
public static final Date TODAYS_DATE = new Date(2003, 1, 8); // January 8, 2003
public static final int CURRENT_QUARTER = 1;
public static final int CURRENT_WEEK = 2;
```

### Rust Implementation

Our Rust implementation faithfully replicates all these bugs to ensure exact compatibility:

```rust
// From date_dim_row_generator.rs

// Replicate quarter sequence bug
let d_quarter_seq = (d_year - 1900) * 4 + d_moy / 3 + 1;

// Replicate weekend days bug
let d_weekend = d_dow == 5 || d_dow == 6;  // Friday or Saturday

// Replicate current day comparison bug
let d_current_day = d_date_sk == TODAYS_DATE.day() as i64;  // Bug: comparing julian to day of month

// From types/date.rs - Replicate leap year bug
pub fn is_leap_year(year: i32) -> bool {
    year % 4 == 0  // Intentionally wrong for compatibility
}
```
