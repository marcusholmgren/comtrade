mod formats;

use crate::parser::TIMESTAMP_MISSING;
use crate::{ComtradeParser, ParseError, ParseResult};
use byteorder::{LittleEndian, ReadBytesExt};
pub use formats::DataFormat;
use std::io::{BufRead, Cursor};

impl<T: BufRead> ComtradeParser<T> {
    pub(super) fn parse_dat(&mut self) -> ParseResult<()> {
        match self.data_format {
            Some(DataFormat::Ascii) => self.parse_dat_ascii(),
            Some(_) => self.parse_dat_binary(),
            None => Err(ParseError::new("Data format not specified.".into())),
        }
    }

    pub(super) fn parse_dat_ascii(&mut self) -> ParseResult<()> {
        // One column for index, one for timestamp.
        let expected_num_cols = (self.num_status_channels + self.num_analog_channels + 2) as usize;

        let mut sample_numbers: Vec<u32> = Vec::with_capacity(self.total_num_samples as usize);
        let mut timestamps: Vec<f64> = Vec::with_capacity(self.total_num_samples as usize);

        for (i, line) in self
            .ascii_dat_contents
            .split('\n')
            .filter(|l| !l.trim().is_empty())
            .enumerate()
        {
            let data_values: Vec<&str> = line.split(',').collect();

            if data_values.len() != expected_num_cols {
                return Err(ParseError::new(format!(
                    "Row {} has incorrect number of columns; expected {} but got {}.",
                    i,
                    expected_num_cols,
                    data_values.len()
                )));
            }

            let sample_number = data_values[0].trim().parse::<u32>().map_err(|_| {
                ParseError::new(format!(
                    "[DAT] Invalid sample number {} on line {}",
                    data_values[0].trim(),
                    i + 1
                ))
            })?;

            sample_numbers.push(sample_number);

            let timestamp = match data_values[1].trim() {
                "" => None, // TODO: Check whether there are any sampling rates. This is critical if there aren't any sampling rates.
                v => Some(v.parse::<u32>().map_err(|_| {
                    ParseError::new(format!(
                        "[DAT] Invalid timestamp {} on line {}.",
                        data_values[1].trim(),
                        i
                    ))
                })?),
            };

            timestamps.push(self.real_time(sample_number, timestamp)?);

            for channel_idx in 0..self.num_analog_channels {
                let col_idx = (channel_idx + 2) as usize;
                let value_str = data_values.get(col_idx).ok_or_else(|| {
                    ParseError::new(format!(
                        "Missing column for analog channel {} on line {}",
                        channel_idx + 1,
                        i + 1
                    ))
                })?.trim();
                let value_raw = value_str.parse::<f64>().map_err(|_| {
                    ParseError::new(format!(
                        "[DAT] Invalid float value {} in analog channel {} on line {}.",
                        value_str,
                        channel_idx + 1,
                        i + 1
                    ))
                })?;

                let adder = self.analog_channels[channel_idx as usize]
                    .config
                    .offset_adder;
                let multiplier = self.analog_channels[channel_idx as usize].config.multiplier;
                let value = value_raw * multiplier + adder;

                self.analog_channels[channel_idx as usize].push_datum(value);
            }

            for channel_idx in 0..self.num_status_channels {
                let col_idx = (channel_idx + self.num_analog_channels + 2) as usize;
                let value_str = data_values.get(col_idx).ok_or_else(|| {
                    ParseError::new(format!(
                        "Missing column for status channel {} on line {}",
                        channel_idx + 1,
                        i + 1
                    ))
                })?.trim();
                let value = value_str.parse::<u8>().map_err(|_| {
                    ParseError::new(format!(
                        "[DAT] Invalid status value {} in status channel {} on line {}",
                        value_str,
                        channel_idx + 1,
                        i + 1
                    ))
                })?;
                self.status_channels[channel_idx as usize].push_datum(value);
            }
        }

        self.builder.sample_numbers(sample_numbers);
        self.builder.timestamps(timestamps);

        Ok(())
    }

    fn parse_dat_binary(&mut self) -> ParseResult<()> {
        // Status channels are binary (0 or 1) and combined into 16-bit bitfields.
        // Each 16-bit bitfield is referred to as a status "group".
        let num_status_groups = (self.num_status_channels as f64 / 16.0).ceil() as usize;

        let mut cursor = Cursor::new(&self.binary_dat_contents);

        let mut sample_numbers: Vec<u32> = Vec::with_capacity(self.total_num_samples as usize);
        let mut timestamps: Vec<f64> = Vec::with_capacity(self.total_num_samples as usize);

        let mut i = 0;
        loop {
            if i >= self.total_num_samples {
                break;
            }

            let sample_number = cursor.read_u32::<LittleEndian>().map_err(|e| {
                ParseError::new(format!("failed to read sample number in binary data: {}", e))
            })?;
            let timestamp = cursor.read_u32::<LittleEndian>().map_err(|e| {
                ParseError::new(format!("failed to read timestamp in binary data: {}", e))
            })?;

            sample_numbers.push(sample_number);
            timestamps.push(self.real_time(
                sample_number,
                if timestamp == TIMESTAMP_MISSING {
                    None
                } else {
                    Some(timestamp)
                },
            )?);

            let mut analog_values = Vec::with_capacity(self.num_analog_channels as usize);
            for channel_idx in 0..self.num_analog_channels {
                let value = match self.data_format {
                    Some(DataFormat::Binary16) => {
                        cursor.read_i16::<LittleEndian>().map_err(|e| {
                            ParseError::new(format!("failed to read binary16 analog value: {}", e))
                        })? as f64
                    }
                    Some(DataFormat::Binary32) => {
                        cursor.read_i32::<LittleEndian>().map_err(|e| {
                            ParseError::new(format!("failed to read binary32 analog value: {}", e))
                        })? as f64
                    }
                    Some(DataFormat::Float32) => {
                        cursor.read_f32::<LittleEndian>().map_err(|e| {
                            ParseError::new(format!("failed to read float32 analog value: {}", e))
                        })? as f64
                    }
                    _ => return Err(ParseError::new("tried to parse binary data for non-binary or invalid data format".to_string())),
                };

                let adder = self.analog_channels[channel_idx as usize]
                    .config
                    .offset_adder;
                let multiplier = self.analog_channels[channel_idx as usize].config.multiplier;
                analog_values.push(value * multiplier + adder);
            }

            for (i, v) in analog_values.into_iter().enumerate() {
                self.analog_channels[i].push_datum(v);
            }

            let mut status_values = Vec::new();
            for _ in 0..num_status_groups {
                let group = cursor.read_u16::<LittleEndian>().map_err(|e| {
                    ParseError::new(format!("failed to read status group: {}", e))
                })?;
                for bit_idx in 0..16 {
                    let bit_mask = 0b01 << bit_idx;
                    let val = (group & bit_mask) >> bit_idx;
                    status_values.push(val as u8);
                }
            }
            let status_values: Vec<u8> = status_values.into_iter().take(self.num_status_channels as usize).collect();

            for (i, v) in status_values.into_iter().enumerate() {
                self.status_channels[i].push_datum(v);
            }

            i += 1;
        }

        self.builder.sample_numbers(sample_numbers);
        self.builder.timestamps(timestamps);

        Ok(())
    }
}
