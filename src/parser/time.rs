use crate::{ComtradeParser, ParseError, ParseResult, SamplingRate};
use chrono::FixedOffset;
use std::io::BufRead;

impl<T: BufRead> ComtradeParser<T> {
    /// Calculate the true value of the timestamp from the in-file value, using the
    /// sampling information if possible, otherwise the in-data timestamp values
    /// along with relevant multiplicative factors from configuration file. This
    /// does *not* include the skew, which needs to be done on a per-channel basis.
    pub(super) fn real_time(&self, sample_number: u32, timestamp: Option<u32>) -> ParseResult<f64> {
        if sample_number == 0 {
            return Err(ParseError::new("sample number cannot be zero".to_string()));
        }

        if !self.is_timestamp_critical {
            let sampling_rate = self.sampling_rate_for_sample(sample_number)?;
            return ParseResult::Ok((sample_number - 1) as f64 / sampling_rate);
        }

        match timestamp {
            Some(ts_value) => {
                let multiplier = self.builder.timestamp_multiplication_factor.unwrap_or(1.0);
                ParseResult::Ok(ts_value as f64 * self.ts_base_unit * multiplier)
            }
            None => ParseResult::Err(ParseError::new(format!(
                "timestamp is critical but not present in sample number {}",
                sample_number
            ))),
        }
    }

    fn sampling_rate_for_sample(&self, sample_number: u32) -> ParseResult<f64> {
        let sampling_rates: &Vec<SamplingRate> =
            self.builder.sampling_rates.as_ref().ok_or_else(|| {
                ParseError::new(
                    "sampling rates not available; cfg file may not have been parsed yet"
                        .to_string(),
                )
            })?;

        let maybe_rate = sampling_rates
            .iter()
            .find(|r| sample_number <= r.end_sample_number);

        match maybe_rate {
            Some(rate) => Ok(rate.rate_hz),
            None => {
                // If the sample number exceeds the maximum defined sampling rate's end_sample_number,
                // fall back to the last known rate if available.
                match sampling_rates.last() {
                    Some(rate) => Ok(rate.rate_hz),
                    None => Err(ParseError::new(format!(
                        "could not determine sampling rate for sample number {}",
                        sample_number
                    ))),
                }
            }
        }
    }
}

/// Parse COMTRADE time offset format into chrono struct.
///
/// COMTRADE format looks like:
///   - "-4" meaning 4 hours west of UTC
///   - "+10h30" meaning 10 hours and 30 minutes east of UTC.
///   - "-7h15" meaning 7 hours and 15 minutes west of UTC.
///   - "0" meaning same as UTC.
///
/// "Not applicable" is a valid value for this, represents in the COMTRADE file
/// as `x` - this is given the value of `None` here.
pub fn parse_time_offset(offset_str: &str) -> ParseResult<Option<FixedOffset>> {
    let time_value = offset_str.trim();

    // Special value indicating offset field does not apply.
    if time_value.to_lowercase() == "x" {
        return Ok(None);
    }

    let maybe_hours = time_value.parse::<i32>();

    if let Ok(hours) = maybe_hours {
        // Offset specified just as number of hours, e.g. "-4", "+10", "0".
        return Ok(Some(FixedOffset::east(hours * 3600)));
    }

    // Offset specified as number + minutes, e.g. "-7h15", "+9h45".
    let time_split: Vec<&str> = time_value.split('h').collect();
    if time_split.len() != 2 {
        return Err(ParseError::new(format!(
            "invalid time offset on line: {}",
            time_value,
        )));
    }

    let hours = time_split[0].trim().parse::<i32>().map_err(|_| {
        ParseError::new(format!(
            "invalid hour offset in time offset: {} in {}",
            time_split[0], time_value,
        ))
    })?;
    let minutes = time_split[1].trim().parse::<i32>().map_err(|_| {
        ParseError::new(format!(
            "invalid minute offset in time offset: {} in {}",
            time_split[1], time_value,
        ))
    })?;

    // Determine the sign of the overall offset from the leading '+'/'-' character,
    // not from the parsed hours value — this handles offsets like "+0h30" (UTC+0:30)
    // where hours is 0 and the sign would otherwise be lost.
    let total_offset = if time_value.starts_with('-') {
        hours * 3600 - minutes * 60
    } else {
        hours * 3600 + minutes * 60
    };

    Ok(Some(FixedOffset::east(total_offset)))
}
