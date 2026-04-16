mod cff;
mod cfg;
mod dat;
mod time;

use std::io::BufRead;
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;

use crate::{Comtrade, ComtradeBuilder, FileType, LeapSecondStatus, TimeQuality};
pub use cfg::{AnalogConfig, AnalogScalingMode, FormatRevision, SamplingRate, StatusConfig};
pub use dat::DataFormat;

const CFG_SEPARATOR: &str = ",";

// To preserve structure integrity, a special value is used in the binary16, binary32
// and float32 data formats when a timestamp is missing.
const TIMESTAMP_MISSING: u32 = 0xffffffff;

pub type ParseResult<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    message: String,
}

impl ParseError {
    pub fn new(message: String) -> Self {
        ParseError { message }
    }
}

impl FromStr for FileType {
    type Err = ParseError;

    fn from_str(value: &str) -> ParseResult<Self> {
        match value.trim().to_lowercase().as_str() {
            "cfg" => Ok(FileType::Cfg),
            "dat" => Ok(FileType::Dat),
            "hdr" => Ok(FileType::Hdr),
            "inf" => Ok(FileType::Inf),
            _ => Err(ParseError::new(format!("invalid file type: '{}'", value))),
        }
    }
}

impl FromStr for DataFormat {
    type Err = ParseError;

    fn from_str(value: &str) -> ParseResult<Self> {
        match value.trim().to_lowercase().as_str() {
            "ascii" => Ok(DataFormat::Ascii),
            "binary" => Ok(DataFormat::Binary16),
            "binary32" => Ok(DataFormat::Binary32),
            "float32" => Ok(DataFormat::Float32),
            _ => Err(ParseError::new(format!(
                "unrecognised or invalid COMTRADE data format: '{}'",
                value.to_owned(),
            ))),
        }
    }
}

impl FromStr for AnalogScalingMode {
    type Err = ParseError;
    fn from_str(value: &str) -> ParseResult<Self> {
        match value.to_lowercase().as_str() {
            "p" => Ok(AnalogScalingMode::Primary),
            "s" => Ok(AnalogScalingMode::Secondary),
            _ => Err(ParseError::new(format!(
                "invalid analog scaling mode: '{}'; must be one of: 's', 'S', 'p', 'P'",
                value,
            ))),
        }
    }
}

impl FromStr for TimeQuality {
    type Err = ParseError;

    fn from_str(value: &str) -> ParseResult<Self> {
        let value_lc = value.to_lowercase();
        let value = value_lc.trim();
        if value.is_empty() || value == "x" {
            return Ok(TimeQuality::Unknown);
        }
        match value {
            "f" => Ok(TimeQuality::ClockFailure),
            "e" => Ok(TimeQuality::ClockUnlocked(4)),
            "d" => Ok(TimeQuality::ClockUnlocked(3)),
            "c" => Ok(TimeQuality::ClockUnlocked(2)),
            "b" => Ok(TimeQuality::ClockUnlocked(1)),
            "a" => Ok(TimeQuality::ClockUnlocked(0)),
            "9" => Ok(TimeQuality::ClockUnlocked(-1)),
            "8" => Ok(TimeQuality::ClockUnlocked(-2)),
            "7" => Ok(TimeQuality::ClockUnlocked(-3)),
            "6" => Ok(TimeQuality::ClockUnlocked(-4)),
            "5" => Ok(TimeQuality::ClockUnlocked(-5)),
            "4" => Ok(TimeQuality::ClockUnlocked(-6)),
            "3" => Ok(TimeQuality::ClockUnlocked(-7)),
            "2" => Ok(TimeQuality::ClockUnlocked(-8)),
            "1" => Ok(TimeQuality::ClockUnlocked(-9)),
            "0" => Ok(TimeQuality::ClockLocked),
            _ => Err(ParseError::new(format!(
                "invalid time quality code '{}'",
                value,
            ))),
        }
    }
}

impl FromStr for LeapSecondStatus {
    type Err = ParseError;

    fn from_str(value: &str) -> ParseResult<Self> {
        let value = value.trim();
        if value.is_empty() || value == "2" {
            return Ok(LeapSecondStatus::Unknown);
        }
        match value {
            "10" => Ok(LeapSecondStatus::HasOccurred),
            "3" => Ok(LeapSecondStatus::NoCapability),
            "1" => Ok(LeapSecondStatus::ToOccur),
            "0" => Ok(LeapSecondStatus::NotPresent),
            _ => Err(ParseError::new(format!(
                "invalid leap second indicator '{}'",
                value,
            ))),
        }
    }
}

lazy_static! {
    static ref CFF_HEADER_REGEXP: Regex = Regex::new(r#"(?i)---\s*file type:\s*(?P<file_type>[a-z]+)(\s+(?P<data_format>[a-z0-9]+))?\s*(:\s*(?P<data_size>\d+))?\s*---$"#).unwrap();
    static ref DATE_REGEXP: Regex = Regex::new("([0-9]{1,2})/([0-9]{1,2})/([0-9]{2,4})").unwrap();
    static ref TIME_REGEXP: Regex = Regex::new("([0-9]{2}):([0-9]{2}):([0-9]{2})(\\.([0-9]{1,12}))?").unwrap();
}

#[cfg(test)]
mod tests {
    use crate::{LeapSecondStatus, TimeQuality};

    #[test]
    fn test_time_quality_parsing() {
        assert_eq!(
            "0".parse::<TimeQuality>().unwrap(),
            TimeQuality::ClockLocked
        );
        assert_eq!(
            "1".parse::<TimeQuality>().unwrap(),
            TimeQuality::ClockUnlocked(-9)
        );
        assert_eq!(
            "9".parse::<TimeQuality>().unwrap(),
            TimeQuality::ClockUnlocked(-1)
        );
        assert_eq!(
            "a".parse::<TimeQuality>().unwrap(),
            TimeQuality::ClockUnlocked(0)
        );
        assert_eq!(
            "b".parse::<TimeQuality>().unwrap(),
            TimeQuality::ClockUnlocked(1)
        );
        assert_eq!(
            "c".parse::<TimeQuality>().unwrap(),
            TimeQuality::ClockUnlocked(2)
        );
        assert_eq!(
            "d".parse::<TimeQuality>().unwrap(),
            TimeQuality::ClockUnlocked(3)
        );
        assert_eq!(
            "e".parse::<TimeQuality>().unwrap(),
            TimeQuality::ClockUnlocked(4)
        );
        assert_eq!(
            "f".parse::<TimeQuality>().unwrap(),
            TimeQuality::ClockFailure
        );
        assert_eq!("x".parse::<TimeQuality>().unwrap(), TimeQuality::Unknown);
        assert_eq!("".parse::<TimeQuality>().unwrap(), TimeQuality::Unknown);
        assert_eq!("  ".parse::<TimeQuality>().unwrap(), TimeQuality::Unknown);
    }

    #[test]
    fn test_leap_second_status_parsing() {
        assert_eq!(
            "0".parse::<LeapSecondStatus>().unwrap(),
            LeapSecondStatus::NotPresent
        );
        assert_eq!(
            "1".parse::<LeapSecondStatus>().unwrap(),
            LeapSecondStatus::ToOccur
        );
        assert_eq!(
            "2".parse::<LeapSecondStatus>().unwrap(),
            LeapSecondStatus::Unknown
        );
        assert_eq!(
            "10".parse::<LeapSecondStatus>().unwrap(),
            LeapSecondStatus::HasOccurred
        );
        assert_eq!(
            "3".parse::<LeapSecondStatus>().unwrap(),
            LeapSecondStatus::NoCapability
        );
        assert_eq!(
            "".parse::<LeapSecondStatus>().unwrap(),
            LeapSecondStatus::Unknown
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnalogChannel {
    pub config: AnalogConfig,
    pub data: Vec<f64>,
}

impl AnalogChannel {
    fn push_datum(&mut self, value: f64) {
        self.data.push(value);
    }

    // TODO: Method for retrieving datum at index / sample number including value and time calculations.
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatusChannel {
    pub config: StatusConfig,
    pub data: Vec<u8>, // Values are 0 or 1.
}

impl StatusChannel {
    fn push_datum(&mut self, value: u8) {
        self.data.push(value);
    }

    // TODO: Method for retrieving datum at index / sample number including time calculations.
}

// Cannot derive builder for this because of complexity of wrapping `T: BufRead` in
// `Option` - I can't figure out how to stop the default implementation from complaining
// that `BufReader<File>` doesn't implement `Copy`.
pub struct ComtradeParserBuilder<T: BufRead> {
    cff_file: Option<T>,
    cfg_file: Option<T>,
    dat_file: Option<T>,
    hdr_file: Option<T>,
    inf_file: Option<T>,
}

impl<T: BufRead> Default for ComtradeParserBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: BufRead> ComtradeParserBuilder<T> {
    pub fn new() -> Self {
        Self {
            cff_file: None,
            cfg_file: None,
            dat_file: None,
            hdr_file: None,
            inf_file: None,
        }
    }

    pub fn cff_file(mut self, file: T) -> Self {
        self.cff_file = Some(file);
        self
    }

    pub fn cfg_file(mut self, file: T) -> Self {
        self.cfg_file = Some(file);
        self
    }

    pub fn dat_file(mut self, file: T) -> Self {
        self.dat_file = Some(file);
        self
    }

    pub fn hdr_file(mut self, file: T) -> Self {
        self.hdr_file = Some(file);
        self
    }

    pub fn inf_file(mut self, file: T) -> Self {
        self.inf_file = Some(file);
        self
    }

    pub fn build(self) -> ComtradeParser<T> {
        ComtradeParser::new(
            self.cff_file,
            self.cfg_file,
            self.dat_file,
            self.hdr_file,
            self.inf_file,
        )
    }
}

pub struct ComtradeParser<T: BufRead> {
    cff_file: Option<T>,
    cfg_file: Option<T>,
    dat_file: Option<T>,
    hdr_file: Option<T>,
    inf_file: Option<T>,

    cfg_contents: String,
    ascii_dat_contents: String,
    binary_dat_contents: Vec<u8>,
    hdr_contents: String,
    inf_contents: String,

    builder: ComtradeBuilder,
    total_num_samples: usize,
    num_analog_channels: usize,
    num_status_channels: usize,
    analog_channels: Vec<AnalogChannel>,
    status_channels: Vec<StatusChannel>,
    is_timestamp_critical: bool,
    ts_base_unit: f64,
    data_format: Option<DataFormat>,
}

impl<T: BufRead> ComtradeParser<T> {
    pub fn new(
        cff_file: Option<T>,
        cfg_file: Option<T>,
        dat_file: Option<T>,
        hdr_file: Option<T>,
        inf_file: Option<T>,
    ) -> Self {
        Self {
            cff_file,
            cfg_file,
            dat_file,
            hdr_file,
            inf_file,

            cfg_contents: String::new(),
            ascii_dat_contents: String::new(),
            binary_dat_contents: vec![],
            hdr_contents: String::new(),
            inf_contents: String::new(),

            builder: ComtradeBuilder::default(),
            total_num_samples: 0,
            num_analog_channels: 0,
            num_status_channels: 0,
            analog_channels: vec![],
            status_channels: vec![],
            is_timestamp_critical: false,
            ts_base_unit: 0.0,
            data_format: None,
        }
    }

    pub fn dat_file(mut self, file: T) -> Self {
        self.dat_file = Some(file);
        self
    }

    pub fn hdr_file(mut self, file: T) -> Self {
        self.hdr_file = Some(file);
        self
    }

    pub fn inf_file(mut self, file: T) -> Self {
        self.inf_file = Some(file);
        self
    }

    pub fn parse(mut self) -> ParseResult<Comtrade> {
        if self.cff_file.is_some() {
            self.load_cff()?;
            self.parse_cfg()
                .map_err(|e| ParseError::new(e.to_string()))?;
            self.parse_dat()?;
        } else {
            if let Some(ref mut cfg_file) = self.cfg_file {
                cfg_file
                    .read_to_string(&mut self.cfg_contents)
                    .map_err(|_| {
                        ParseError::new("unable to read specified .cfg file".to_string())
                    })?;
            } else {
                return Err(ParseError::new(
                    "you must specify either .cff or .cfg file".to_string(),
                ));
            }

            self.parse_cfg()
                .map_err(|e| ParseError::new(e.to_string()))?;

            if let Some(ref mut dat_file) = self.dat_file {
                match self.data_format {
                    Some(DataFormat::Ascii) => {
                        dat_file
                            .read_to_string(&mut self.ascii_dat_contents)
                            .map_err(|_| {
                                ParseError::new("unable to read specified .dat file".into())
                            })?;
                    }
                    None => {
                        return Err(ParseError::new("unknown data format for data file.".into()));
                    }
                    // Other binary format.
                    _ => {
                        dat_file
                            .read_to_end(&mut self.binary_dat_contents)
                            .map_err(|_| {
                                ParseError::new("unable to read specified .dat file".into())
                            })?;
                    }
                }
            } else {
                return Err(ParseError::new(
                    "you must specify either .cff or .dat file".to_string(),
                ));
            }

            self.parse_dat()?;

            if let Some(ref mut hdr_file) = self.hdr_file {
                hdr_file
                    .read_to_string(&mut self.hdr_contents)
                    .map_err(|_| {
                        ParseError::new("unable to read specified .hdr file".to_string())
                    })?;
            }

            if let Some(ref mut inf_file) = self.inf_file {
                inf_file
                    .read_to_string(&mut self.inf_contents)
                    .map_err(|_| {
                        ParseError::new("unable to read specified .inf file".to_string())
                    })?;
            }
        }

        // `.hdr` and `.inf` files don't need parsing - if present they're
        // non-machine-readable text files for reference for humans to look at.

        self.builder.analog_channels(self.analog_channels);
        self.builder.status_channels(self.status_channels);

        self.builder
            .build()
            .map_err(|e| ParseError::new(e.to_string()))
    }
}
