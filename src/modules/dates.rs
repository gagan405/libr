/// Porting of https://www.benjoffe.com/fast-date-64

/// Very Fast 64-Bit Date Algorithm (x64 version)
/// Converts a day count to a calendar date (year, month, day)
/// Epoch: January 1, 1970 (Unix epoch) = Day 0
#[derive(Debug, PartialEq, Clone)]
pub struct Date {
    pub year: i64,
    pub month: u32,
    pub day: u32,
}

pub fn days_to_date(days: i64) -> Date {
    const ERAS: i64 = 4726498270;
    const D_SHIFT: i64 = 146097 * ERAS - 719469;
    const Y_SHIFT: i64 = 400 * ERAS - 1;
    const C1: u64 = 505054698555331;      // floor(2^64 * 4 / 146097)
    const C2: u64 = 50504432782230121;    // ceil(2^64 * 4 / 1461)
    const C3: u64 = 8619973866219416;     // floor(2^64 / 2140)

    // Step 1: Adjust for 100/400 leap year rule
    let rev = D_SHIFT - days;

    // Get upper 64 bits of rev * C1
    let cen = (((rev as u128) * (C1 as u128)) >> 64) as i64;
    let jul = rev + cen - cen / 4;

    // Step 2: Determine year and year-part
    let num = (jul as u128) * (C2 as u128);
    let yrs = Y_SHIFT - ((num >> 64) as i64);
    let low = num as u64;  // Lower 64 bits

    // Get upper 64 bits of low * 782432
    let ypt = (((low as u128) * 782432u128) >> 64) as i64;

    // Step 3: Determine if January or February
    let bump = ypt < 126464;
    let shift = if bump { 191360 } else { 977792 };

    // Step 4: Year-modulo-bitshift for leap years
    let n = (yrs % 4) * 512 + shift - ypt;

    // Get upper 64 bits of (n % 65536) * C3
    let d = ((((n % 65536) as u128) * (C3 as u128)) >> 64) as u32;

    let day = d + 1;
    let month = (n / 65536) as u32;
    let year = yrs + if bump { 1 } else { 0 };

    Date { year, month, day }
}

impl Date {
    /// Check if a year is a leap year (Gregorian calendar rules)
    pub fn is_leap_year(year: i64) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    /// Get the number of days in a given month for a given year
    pub fn days_in_month(year: i64, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => if Self::is_leap_year(year) { 29 } else { 28 },
            _ => 0,
        }
    }

    /// Convert date back to days since Unix epoch (inverse operation)
    /// This is a reference implementation for testing
    pub fn to_days(&self) -> i64 {
        if self.month < 1 || self.month > 12 || self.day < 1 {
            return 0; // Invalid date
        }

        // Days from epoch (1970-01-01) to this date
        let mut days = 0i64;

        // Add/subtract complete years from 1970
        if self.year >= 1970 {
            for y in 1970..self.year {
                days += if Self::is_leap_year(y) { 366 } else { 365 };
            }
        } else {
            for y in self.year..1970 {
                days -= if Self::is_leap_year(y) { 366 } else { 365 };
            }
        }

        // Add days for complete months in current year
        for m in 1..self.month {
            days += Self::days_in_month(self.year, m) as i64;
        }

        // Add remaining days (subtract 1 because day 1 = 0 days offset)
        days += self.day as i64 - 1;

        days
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_epoch() {
        let date = days_to_date(0);
        assert_eq!(date, Date { year: 1970, month: 1, day: 1 });
    }

    #[test]
    fn test_day_before_epoch() {
        let date = days_to_date(-1);
        assert_eq!(date, Date { year: 1969, month: 12, day: 31 });
    }

    #[test]
    fn test_known_dates() {
        // Jan 1, 2000 = 10957 days after Unix epoch
        let date = days_to_date(10957);
        assert_eq!(date, Date { year: 2000, month: 1, day: 1 });

        // Feb 29, 2000 (leap year)
        let date = days_to_date(11016);
        assert_eq!(date, Date { year: 2000, month: 2, day: 29 });
    }

    #[test]
    fn test_inverse_function() {
        let date = Date { year: 2000, month: 1, day: 1 };
        let days = date.to_days();
        assert_eq!(days, 10957);

        let date2 = Date { year: 1970, month: 1, day: 1 };
        assert_eq!(date2.to_days(), 0);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Test that month is always in valid range [1, 12]
        #[test]
        fn month_always_valid(days in -100000i64..1000000i64) {
            let date = days_to_date(days);
            prop_assert!(date.month >= 1 && date.month <= 12,
                        "Month {} out of range for day {}", date.month, days);
        }

        /// Test that day is always in valid range [1, 31]
        #[test]
        fn day_always_positive_and_reasonable(days in -100000i64..1000000i64) {
            let date = days_to_date(days);
            prop_assert!(date.day >= 1 && date.day <= 31,
                        "Day {} out of range for day {}", date.day, days);
        }

        /// Test that day is valid for the given month
        #[test]
        fn day_valid_for_month(days in -100000i64..1000000i64) {
            let date = days_to_date(days);
            let max_days = Date::days_in_month(date.year, date.month);
            prop_assert!(date.day <= max_days,
                        "Day {} exceeds max {} for month {} in year {}",
                        date.day, max_days, date.month, date.year);
        }

        /// Test that consecutive days produce valid date sequences
        #[test]
        fn consecutive_days_are_sequential(days in -50000i64..50000i64) {
            let date1 = days_to_date(days);
            let date2 = days_to_date(days + 1);

            // Either same month with day+1, or next month/year
            let same_month = date1.year == date2.year &&
                           date1.month == date2.month &&
                           date2.day == date1.day + 1;

            let next_month = date1.year == date2.year &&
                           date2.month == date1.month + 1 &&
                           date2.day == 1;

            let next_year = date2.year == date1.year + 1 &&
                          date2.month == 1 &&
                          date2.day == 1 &&
                          date1.month == 12;

            prop_assert!(same_month || next_month || next_year,
                        "Days {} and {} produced non-sequential dates: {:?} -> {:?}",
                        days, days + 1, date1, date2);
        }

        /// Test roundtrip: days -> date -> days
        #[test]
        fn roundtrip_conversion(days in -50000i64..50000i64) {
            let date = days_to_date(days);
            let days_back = date.to_days();
            prop_assert_eq!(days, days_back,
                           "Roundtrip failed: {} -> {:?} -> {}",
                           days, date, days_back);
        }

        /// Test that February 29 only appears in leap years
        #[test]
        fn feb_29_only_in_leap_years(days in -100000i64..1000000i64) {
            let date = days_to_date(days);

            if date.month == 2 && date.day == 29 {
                prop_assert!(Date::is_leap_year(date.year),
                            "Feb 29 appeared in non-leap year {} for day {}",
                            date.year, days);
            }
        }

        /// Test monotonicity: more days = later or equal date
        #[test]
        fn monotonic_dates(days1 in -10000i64..10000i64, offset in 1u32..1000u32) {
            let days2 = days1 + offset as i64;
            let date1 = days_to_date(days1);
            let date2 = days_to_date(days2);

            let is_later = date2.year > date1.year ||
                          (date2.year == date1.year && date2.month > date1.month) ||
                          (date2.year == date1.year && date2.month == date1.month && date2.day >= date1.day);

            prop_assert!(is_later,
                        "Day {} ({:?}) should be >= day {} ({:?})",
                        days2, date2, days1, date1);
        }

        /// Test century leap year rules (1900 not leap, 2000 is leap)
        #[test]
        fn century_leap_rules_in_output(days in -100000i64..1000000i64) {
            let date = days_to_date(days);

            // If this is a century year and February 29
            if date.year % 100 == 0 && date.month == 2 && date.day == 29 {
                prop_assert!(date.year % 400 == 0,
                            "Feb 29 in century year {} that's not divisible by 400",
                            date.year);
            }
        }

        /// Test that adding exactly 1 day moves forward correctly
        #[test]
        fn one_day_increment(days in -50000i64..50000i64) {
            let date1 = days_to_date(days);
            let date2 = days_to_date(days + 1);

            // Calculate total ordering for dates
            let ord1 = date1.year * 10000 + date1.month as i64 * 100 + date1.day as i64;
            let ord2 = date2.year * 10000 + date2.month as i64 * 100 + date2.day as i64;

            prop_assert!(ord2 > ord1,
                        "Date should strictly increase: {:?} -> {:?}",
                        date1, date2);
        }

        /// Test known historical dates
        #[test]
        fn test_year_2000_range(day_offset in 0i64..366) {
            // Year 2000 started at day 10957
            let days = 10957 + day_offset;
            let date = days_to_date(days);
            prop_assert_eq!(date.year, 2000, "Day {} should be in year 2000", days);
        }
    }
}
