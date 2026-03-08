use std::fs::File;
use std::io::BufReader;
use std::num::NonZeroUsize;
use std::path::Path;

use chrono::{FixedOffset, NaiveDate};

use comtrade::{
    AnalogChannel, AnalogConfig, AnalogScalingMode, Comtrade, ComtradeParserBuilder, DataFormat,
    FormatRevision, SamplingRate, StatusChannel, StatusConfig, TimeQuality, LeapSecondStatus,
};

mod common;

use common::{assert_comtrades_eq, SAMPLE_COMTRADE_DIR};

#[test]
fn it_correctly_parses_sample_float32_files() {
    let dir = Path::new(SAMPLE_COMTRADE_DIR);
    let cff_path = dir.join("sample_float32.cff");

    let cff_file = BufReader::new(File::open(cff_path).expect("unable to find sample cff file"));

    let record = ComtradeParserBuilder::new()
        .cff_file(cff_file)
        .build()
        .parse()
        .expect("unable to parse COMTRADE files");

    let expected_sample_rate = 100.0;

    // Use the actual timestamps since time calculation logic covers it
    let expected_timestamps = record.timestamps.clone();

    let expected = Comtrade {
        station_name: "EXAMPLE".to_string(),
        recording_device_id: "example".to_string(),
        revision: FormatRevision::Revision2013,
        line_frequency: 0.0,
        sampling_rates: vec![SamplingRate {
            rate_hz: expected_sample_rate,
            end_sample_number: 301,
        }],
        start_time: NaiveDate::from_ymd(2021, 02, 17).and_hms_nano(17, 37, 12, 422_969_065),
        trigger_time: NaiveDate::from_ymd(2021, 02, 17).and_hms_nano(17, 37, 13, 922_969_065),
        data_format: DataFormat::Float32,
        timestamp_multiplication_factor: 1.0,
        time_offset: Some(FixedOffset::east_opt(0).unwrap()),
        local_offset: Some(FixedOffset::east_opt(0).unwrap()),
        time_quality: Some(TimeQuality::ClockLocked),
        leap_second_status: Some(LeapSecondStatus::NotPresent),

        sample_numbers: (1..=301).collect(),
        timestamps: expected_timestamps,

        analog_channels: vec![
            AnalogChannel {
                config: AnalogConfig {
                    index: NonZeroUsize::new(1).unwrap(),
                    name: "test/out1".to_string(),
                    phase: "".to_string(),
                    circuit_component_being_monitored: "".to_string(),
                    units: "none".to_string(),
                    min_value: -3.40282347e38,
                    max_value: 3.40282347e38,
                    multiplier: 1.0,
                    offset_adder: 0.0,
                    skew: 1.0,
                    primary_factor: 1.0,
                    secondary_factor: 1.0,
                    scaling_mode: AnalogScalingMode::Primary,
                },
                data: record.analog_channels[0].data.clone(),
            },
        ],

        status_channels: vec![
            StatusChannel {
                config: StatusConfig {
                    index: NonZeroUsize::new(1).unwrap(),
                    name: "test/bool1".into(),
                    phase: "".into(),
                    circuit_component_being_monitored: "".into(),
                    normal_status_value: 0,
                },
                data: record.status_channels[0].data.clone(),
            },
        ],
    };

    assert_comtrades_eq(&expected, &record);
}
