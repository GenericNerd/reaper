use lazy_static::lazy_static;
use regex::Regex;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct Duration {
    pub years: i64,
    pub months: i64,
    pub weeks: i64,
    pub days: i64,
    pub hours: i64,
    pub minutes: i64,
    pub seconds: i64,
    pub permanent: bool,
    pub string: String,
}

impl Duration {
    pub fn new(string: &str) -> Duration {
        debug!("Parsing duration: {}", string);
        let mut duration = Duration {
            years: 0,
            months: 0,
            weeks: 0,
            days: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
            permanent: false,
            string: string.to_string().to_lowercase(),
        };

        lazy_static! {
            static ref DURATION_REGEX: Regex = Regex::new(r"(\d+)\S*(y|mo|w|d|h|m|s)").unwrap();
        }

        for capture in DURATION_REGEX.captures_iter(&string.to_lowercase()) {
            let value = match capture.get(1) {
                Some(value) => match value.as_str().parse::<i64>() {
                    Ok(value) => value,
                    Err(_) => return duration,
                },
                None => return duration,
            };

            let unit = match capture.get(2) {
                Some(unit) => unit.as_str(),
                None => return duration,
            };

            match unit {
                "y" => duration.years = value,
                "mo" => duration.months = value,
                "w" => duration.weeks = value,
                "d" => duration.days = value,
                "h" => duration.hours = value,
                "m" => duration.minutes = value,
                "s" => duration.seconds = value,
                _ => {}
            }
        }

        debug!("Parsed duration: {:?}", duration);
        duration
    }

    pub fn permanent() -> Duration {
        Duration {
            years: 0,
            months: 0,
            weeks: 0,
            days: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
            permanent: true,
            string: String::new(),
        }
    }

    pub fn to_timestamp(&self) -> Option<time::OffsetDateTime> {
        if self.permanent {
            return None;
        }

        time::OffsetDateTime::now_utc().checked_add(time::Duration::new(
            self.seconds
                + (self.minutes * 60)
                + (self.hours * 60 * 60)
                + (self.days * 60 * 60 * 24)
                + (self.weeks * 60 * 60 * 24 * 7)
                + (self.months * 60 * 60 * 24 * 30)
                + (self.years * 60 * 60 * 24 * 365),
            0,
        ))
    }
}
