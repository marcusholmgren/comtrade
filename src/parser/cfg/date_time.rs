use crate::error::ComtradeError;
use crate::parser::cfg::ConfigLine;
use crate::FormatRevision;
use chrono::{NaiveDateTime, NaiveTime};
use std::cmp::Ordering;

/// The expected precision of the timestamps.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum TimePrecision {
    Microseconds,
    Nanoseconds,
}

impl TimePrecision {
    pub fn to_value(self) -> f64 {
        match self {
            TimePrecision::Microseconds => 1E-6,
            TimePrecision::Nanoseconds => 1E-9,
        }
    }
}

impl Ord for TimePrecision {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (TimePrecision::Microseconds, TimePrecision::Nanoseconds) => Ordering::Greater,
            (TimePrecision::Nanoseconds, TimePrecision::Microseconds) => Ordering::Less,
            _ => Ordering::Equal,
        }
    }
}

impl PartialOrd for TimePrecision {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The timestamp and precision.
pub struct ComtradeDateTime {
    pub date_time: NaiveDateTime,
    pub precision: TimePrecision,
}

impl ComtradeDateTime {
    pub fn from_config_line<'a>(
        mut line: impl ConfigLine<'a>,
        revision: &FormatRevision,
    ) -> Result<ComtradeDateTime, ComtradeError> {
        let date_string = line.read_next()?;
        let time_string = line.read_next()?;
        let date = revision.read_date(date_string)?;
        let time = NaiveTime::parse_from_str(time_string, "%H:%M:%S%.f").map_err(|_| {
            ComtradeError::InvalidValue {
                value: time_string.to_string(),
                type_: "time",
                field: "time",
            }
        })?;
        let precision = ts_base_unit(time_string)?;
        Ok(ComtradeDateTime {
            date_time: NaiveDateTime::new(date, time),
            precision,
        })
    }
}

/// If a timestamp is specified to 6 dp then the timestamps should be interpreted as
/// in the base unit of microseconds. If the timestamp has 9 dp, the timestamps should
/// be interpreted in nanoseconds.
pub fn ts_base_unit(datetime_stamp: &str) -> Result<TimePrecision, ComtradeError> {
    let mut split_at_period = datetime_stamp.rsplit('.');
    let last_section = split_at_period
        .next()
        .ok_or(ComtradeError::CantFindTimestampPrecision)?;

    // Only have 1 section.
    if split_at_period.next().is_none() {
        return Err(ComtradeError::CantFindTimestampPrecision);
    }

    if last_section.len() <= 6 {
        Ok(TimePrecision::Microseconds)
    } else {
        Ok(TimePrecision::Nanoseconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::cfg::split_cfg_line;
    use crate::FormatRevision;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn read_date_with_revision_1991() {
        let revision = FormatRevision::Revision1991;
        let line = "12/31/1999,12:31:59.123456";
        let line = split_cfg_line(line);
        let time = ComtradeDateTime::from_config_line(line, &revision).unwrap();
        assert_eq!(
            time.date_time,
            NaiveDateTime::new(
                NaiveDate::from_ymd(1999, 12, 31),
                NaiveTime::from_hms_micro(12, 31, 59, 123456)
            )
        );
    }

    #[test]
    fn read_date_with_revision_1999() {
        let revision = FormatRevision::Revision1999;
        let line = "31/12/1999,12:31:59.123456";
        let line = split_cfg_line(line);
        let time = ComtradeDateTime::from_config_line(line, &revision).unwrap();
        assert_eq!(
            time.date_time,
            NaiveDateTime::new(
                NaiveDate::from_ymd(1999, 12, 31),
                NaiveTime::from_hms_micro(12, 31, 59, 123456)
            )
        );
    }

    #[test]
    fn read_date_with_revision_2013() {
        let revision = FormatRevision::Revision2013;
        let line = "31/12/1999,12:31:59.123456";
        let line = split_cfg_line(line);
        let time = ComtradeDateTime::from_config_line(line, &revision).unwrap();
        assert_eq!(
            time.date_time,
            NaiveDateTime::new(
                NaiveDate::from_ymd(1999, 12, 31),
                NaiveTime::from_hms_micro(12, 31, 59, 123456)
            )
        );
    }

    #[test]
    fn read_time_precision_microseconds_6_digits_or_less() {
        let revision = FormatRevision::Revision2013;
        let line = "31/12/1999,12:31:59.123456";
        let line = split_cfg_line(line);
        let time = ComtradeDateTime::from_config_line(line, &revision).unwrap();
        assert_eq!(time.precision, TimePrecision::Microseconds);

        let line = "31/12/1999,12:31:59.12345";
        let line = split_cfg_line(line);
        let time = ComtradeDateTime::from_config_line(line, &revision).unwrap();
        assert_eq!(time.precision, TimePrecision::Microseconds);

        let line = "31/12/1999,12:31:59.123";
        let line = split_cfg_line(line);
        let time = ComtradeDateTime::from_config_line(line, &revision).unwrap();
        assert_eq!(time.precision, TimePrecision::Microseconds);
    }

    #[test]
    fn read_time_precision_nanoseconds_7_digits_or_more() {
        let revision = FormatRevision::Revision2013;
        let line = "31/12/1999,12:31:59.1234567";
        let line = split_cfg_line(line);
        let time = ComtradeDateTime::from_config_line(line, &revision).unwrap();
        assert_eq!(time.precision, TimePrecision::Nanoseconds);

        let line = "31/12/1999,12:31:59.12345678";
        let line = split_cfg_line(line);
        let time = ComtradeDateTime::from_config_line(line, &revision).unwrap();
        assert_eq!(time.precision, TimePrecision::Nanoseconds);

        let line = "31/12/1999,12:31:59.123456789";
        let line = split_cfg_line(line);
        let time = ComtradeDateTime::from_config_line(line, &revision).unwrap();
        assert_eq!(time.precision, TimePrecision::Nanoseconds);
    }

    #[test]
    fn read_time_precision_nanoseconds_error_if_no_fractional_digits() {
        let revision = FormatRevision::Revision2013;
        let line = "31/12/1999,12:31:59";
        let line = split_cfg_line(line);
        let time = ComtradeDateTime::from_config_line(line, &revision);
        assert!(matches!(
            time,
            Err(ComtradeError::CantFindTimestampPrecision)
        ));
    }

    #[test]
    fn time_precision_ordering() {
        assert!(TimePrecision::Microseconds > TimePrecision::Nanoseconds);
    }

    #[test]
    fn test_precision_to_value() {
        assert_eq!(TimePrecision::Microseconds.to_value(), 1E-6);
        assert_eq!(TimePrecision::Nanoseconds.to_value(), 1E-9);
    }
}
