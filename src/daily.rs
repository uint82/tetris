use std::time::{SystemTime, UNIX_EPOCH};

const SECONDS_PER_DAY: u64 = 86_400;
const SEED_SALT: u64 = 0xD1B5_4A32_D192_ED03;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DailyDate {
    days: i64,
}

impl DailyDate {
    pub fn today() -> Self {
        let days = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs() / SECONDS_PER_DAY);
        Self { days: days as i64 }
    }

    pub const fn from_days(days: i64) -> Self {
        Self { days }
    }

    pub const fn days(self) -> i64 {
        self.days
    }

    pub const fn seed(self) -> u64 {
        (self.days as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ SEED_SALT
    }

    pub fn iso(self) -> String {
        let (year, month, day) = civil_from_days(self.days);
        format!("{year:04}-{month:02}-{day:02}")
    }
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = if month <= 2 { year + 1 } else { year };
    (year, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_epoch_days_to_dates() {
        assert_eq!(DailyDate::from_days(0).iso(), "1970-01-01");
        assert_eq!(DailyDate::from_days(10_957).iso(), "2000-01-01");
        assert_eq!(DailyDate::from_days(19_782).iso(), "2024-02-29");
        assert_eq!(DailyDate::from_days(20_751).iso(), "2026-10-25");
    }

    #[test]
    fn seeds_are_stable_and_date_specific() {
        let date = DailyDate::from_days(20_751);
        assert_eq!(date.seed(), DailyDate::from_days(20_751).seed());
        assert_ne!(date.seed(), DailyDate::from_days(20_750).seed());
    }
}
