use crate::error::ComtradeError;
use chrono::NaiveDate;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FormatRevision {
    #[default]
    Revision1991,
    Revision1999,
    Revision2013,
}

impl FromStr for FormatRevision {
    type Err = ComtradeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "1991" => Ok(FormatRevision::Revision1991),
            "1999" => Ok(FormatRevision::Revision1999),
            "2013" => Ok(FormatRevision::Revision2013),
            _ => Err(ComtradeError::BadRevisionFormat(value.to_string())),
        }
    }
}

impl FormatRevision {
    pub fn read_date(&self, date: &str) -> Result<NaiveDate, ComtradeError> {
        // 1991 revision uses mm/dd/yyyy format for date whereas 1999 and 2013 use dd/mm/yyyy.
        let format = match self {
            FormatRevision::Revision1991 => "%m/%d/%Y",
            FormatRevision::Revision1999 | FormatRevision::Revision2013 => "%d/%m/%Y",
        };
        NaiveDate::parse_from_str(date, format).map_err(|_| ComtradeError::InvalidValue {
            value: date.to_string(),
            type_: "date",
            field: "date",
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_revision_from_str_works() {
        assert_eq!(
            FormatRevision::from_str("1991").unwrap(),
            FormatRevision::Revision1991
        );
        assert_eq!(
            FormatRevision::from_str("1999").unwrap(),
            FormatRevision::Revision1999
        );
        assert_eq!(
            FormatRevision::from_str("2013").unwrap(),
            FormatRevision::Revision2013
        );
    }

    #[test]
    fn error_in_format_revision_from_str() {
        assert!(FormatRevision::from_str("1990").is_err());
    }
}
