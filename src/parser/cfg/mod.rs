mod analog_channels;
mod date_time;
mod id_line;
mod revisions;
mod sample_rates;
mod status_channel;

use crate::error::ComtradeError;
use crate::parser::cfg::id_line::IdRow;
use crate::parser::time::parse_time_offset;
use crate::parser::{AnalogChannel, DataFormat, StatusChannel, CFG_SEPARATOR};
use crate::ComtradeParser;
pub use analog_channels::{AnalogConfig, AnalogScalingMode};
use date_time::ComtradeDateTime;
pub use revisions::FormatRevision;
pub use sample_rates::SamplingRate;
pub use status_channel::StatusConfig;
use std::any::type_name;
use std::io::BufRead;
use std::str::FromStr;

impl<T: BufRead> ComtradeParser<T> {
    pub(super) fn parse_cfg(&mut self) -> Result<(), ComtradeError> {
        // TODO: There must be a more efficient way of doing this using line iterators,
        //  I just need to figure out how to create my own line iterator in the
        //  `load_cff()` function.
        let mut lines = self.cfg_contents.split('\n');

        let early_end_err = || ComtradeError::UnexpectedEndOfCfgFile;

        let line = lines.next().ok_or_else(early_end_err)?;
        let line = split_cfg_line(line);
        let id_line = IdRow::from_config_line(line)?;

        self.builder.station_name(id_line.station_name);
        self.builder
            .recording_device_id(id_line.recording_device_id);
        self.builder.revision(id_line.format_revision);
        let format_revision = id_line.format_revision;

        let line = lines.next().ok_or_else(early_end_err)?;
        let ChannelSizes {
            analog: num_analog_channels,
            status: num_status_channels,
        } = ChannelSizes::from_line(split_cfg_line(line))?;
        self.num_analog_channels = num_analog_channels;
        self.num_status_channels = num_status_channels;

        let mut analog_channels: Vec<AnalogConfig> =
            Vec::with_capacity(self.num_analog_channels);
        let mut status_channels: Vec<StatusConfig> =
            Vec::with_capacity(self.num_status_channels);

        for _ in 0..num_analog_channels {
            let line = lines.next().ok_or_else(early_end_err)?;
            let config_line = split_cfg_line(line);
            analog_channels.push(AnalogConfig::from_cfg_row(config_line)?);
        }

        for _ in 0..num_status_channels {
            let line = lines.next().ok_or_else(early_end_err)?;
            let config_line = split_cfg_line(line);
            status_channels.push(StatusConfig::from_config_row(config_line)?);
        }
        self.analog_channels = analog_channels
            .into_iter()
            .map(|c| AnalogChannel {
                config: c,
                data: Vec::new(),
            })
            .collect();
        self.status_channels = status_channels
            .into_iter()
            .map(|c| StatusChannel {
                config: c,
                data: Vec::new(),
            })
            .collect();

        let line = lines.next().ok_or_else(early_end_err)?;
        let mut line = split_cfg_line(line);

        // Line frequency
        // lf
        let line_frequency = line.read_value()?;
        self.builder.line_frequency(line_frequency);

        let line = lines.next().ok_or_else(early_end_err)?;
        let mut line = split_cfg_line(line);
        let num_sampling_rates = line.read_value()?;

        let mut sampling_rates: Vec<SamplingRate> = Vec::with_capacity(num_sampling_rates as usize);

        for _ in 0..num_sampling_rates {
            let line = lines.next().ok_or_else(early_end_err)?;
            let line = split_cfg_line(line);
            let sampling_rate = SamplingRate::from_config_line(line)?;
            sampling_rates.push(sampling_rate);
        }

        // If file has 0 for number of sample rates, there's an extra line which just contains 0
        // indicating no fixed sample rate and the total number of samples.
        if num_sampling_rates == 0 {
            let line = lines.next().ok_or_else(early_end_err)?;
            let mut line = split_cfg_line(line);
            // Ignore the first value (0)
            let _ = line.read_value::<u32>()?;
            let endsamp: u32 = line.read_value()?;
            self.total_num_samples = endsamp as usize;
        } else {
            self.total_num_samples = sampling_rates
                .iter()
                .map(|r| r.end_sample_number)
                .max()
                .unwrap_or(0) as usize;
        }

        self.is_timestamp_critical = num_sampling_rates == 0;
        self.builder.sampling_rates(sampling_rates);

        let line = lines.next().ok_or_else(early_end_err)?.trim();
        let start_time =
            ComtradeDateTime::from_config_line(split_cfg_line(line), &format_revision)?;

        self.builder.start_time(start_time.date_time);
        self.ts_base_unit = start_time.precision.to_value();

        let line = lines.next().ok_or_else(early_end_err)?.trim();

        // Time that the COMTRADE record recording was triggered.
        let trigger_time =
            ComtradeDateTime::from_config_line(split_cfg_line(line), &format_revision)?;
        self.builder.trigger_time(trigger_time.date_time);

        // According to the spec, if the start time is in micro/nanoseconds, the
        // other one should be too. If they are inconsistent, just take the lower one
        // to be safe. In the future this would be a good place to raise a warning.
        self.ts_base_unit = self.ts_base_unit.min(trigger_time.precision.to_value());

        let line = lines.next().ok_or_else(early_end_err)?;
        let mut line = split_cfg_line(line);

        // Data file type
        // ft
        let data_format: DataFormat = line.read_value()?;
        self.data_format = Some(data_format.clone());
        self.builder.data_format(data_format);

        // 1991 format ends here - rest of values are 1999 and 2013 only.
        if format_revision == FormatRevision::Revision1991 {
            return Ok(());
        }

        let line = lines.next().ok_or_else(early_end_err)?;
        let mut line_values = split_cfg_line(line);

        // Time stamp multiplication factor
        // timemult
        // The base unit for the timestamps in the data file is determined from the CFG,
        // apparently from the time/stamp. It's not clear to me how this is determined.
        // Regardless, this multiplicative factor allows you to store longer time ranges
        // within a single COMTRADE record.

        let time_mult = line_values.read_value()?;
        self.builder.timestamp_multiplication_factor(time_mult);

        // Default values for optional revision-based fields.
        self.builder.time_offset(None);
        self.builder.local_offset(None);
        self.builder.time_quality(None);
        self.builder.leap_second_status(None);

        // 1999 format ends here - rest of values are 2013 only.
        if format_revision == FormatRevision::Revision1999 {
            return Ok(());
        }

        let line = lines.next().ok_or_else(early_end_err)?;
        let mut line_values = split_cfg_line(line);
        let time_offset = parse_time_offset(&line_values.read_value::<String>()?)
            .map_err(ComtradeError::ParserError)?;
        let local_offset = parse_time_offset(&line_values.read_value::<String>()?)
            .map_err(ComtradeError::ParserError)?;

        // Time information and relationship between local time and UTC
        // time_code, local_code
        self.builder.time_offset(time_offset);
        self.builder.local_offset(local_offset);

        let line = lines.next().ok_or_else(early_end_err)?;
        let mut line_values = split_cfg_line(line);

        // Time quality of samples
        // tmq_code,leapsec
        let tmq_code = line_values.read_value()?;
        self.builder.time_quality(Some(tmq_code));

        let leap_second_status = line_values.read_value()?;
        self.builder.leap_second_status(Some(leap_second_status));

        Ok(())
    }
}

/// Implement a line as a trait alias for clearer implementation.
pub trait ConfigLine<'a>: Iterator<Item = &'a str> {
    /// Read the next value as a str type.
    ///
    /// This will assume there should be a next value and return in an error.
    fn read_next(&mut self) -> Result<&'a str, ComtradeError> {
        self.next()
            .ok_or(ComtradeError::MissingLineElements(""))
            .map(|s| s.trim())
    }
    /// Read the next value as any parsable type.
    fn read_value<T: FromStr>(&mut self) -> Result<T, ComtradeError> {
        let str_value = self.read_next()?;
        str_value.parse().map_err(|_| ComtradeError::InvalidValue {
            value: str_value.to_string(),
            type_: type_name::<T>(),
            field: "",
        })
    }

    /// This is used when there is an additional character on the end.
    /// For example 16A for channels where we want 16.
    fn read_value_with_trailing_char<T: FromStr>(&mut self) -> Result<T, ComtradeError> {
        let str_value = self.read_next()?;
        let trimmed_value = &str_value[..str_value.len().saturating_sub(1)];
        trimmed_value
            .parse()
            .map_err(|_| ComtradeError::InvalidValue {
                value: str_value.to_string(),
                type_: type_name::<T>(),
                field: "",
            })
    }
}
/// Broad implementation of this trait so it acts as an alias.
impl<'a, T: Iterator<Item = &'a str>> ConfigLine<'a> for T {}

fn split_cfg_line(line: &str) -> impl ConfigLine<'_> {
    line.split(CFG_SEPARATOR).map(|s| s.trim())
}

struct ChannelSizes {
    analog: usize,
    status: usize,
}

impl ChannelSizes {
    fn from_line<'a>(mut cfg_line: impl ConfigLine<'a>) -> Result<Self, ComtradeError> {
        // Total is not actually needed.
        let _total: usize = cfg_line
            .read_value()
            .map_err(|e| e.add_context("Channel Sizes: Total"))?;
        let analog = cfg_line
            .read_value_with_trailing_char()
            .map_err(|e| e.add_context("Channel Sizes: Analog"))?;
        let status = cfg_line
            .read_value_with_trailing_char()
            .map_err(|e| e.add_context("Channel Sizes: Status"))?;
        Ok(Self { analog, status })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_line_uses_csv_seperator_and_trims_whitespace() {
        let line = "  1, 2, 3 ,4   ";
        let mut iter = split_cfg_line(line);
        assert_eq!(iter.next(), Some("1"));
        assert_eq!(iter.next(), Some("2"));
        assert_eq!(iter.next(), Some("3"));
        assert_eq!(iter.next(), Some("4"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn get_channel_counts() {
        let line = "20,4A,16D ";
        let line = split_cfg_line(line);
        let sizes = ChannelSizes::from_line(line).unwrap();
        assert_eq!(sizes.analog, 4);
        assert_eq!(sizes.status, 16);
    }
}
