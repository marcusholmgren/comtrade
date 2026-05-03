use std::fs::File;
use std::io::BufReader;
use std::num::NonZeroUsize;
use std::path::Path;

use chrono::{FixedOffset, NaiveDate};

use comtrade::{
    AnalogChannel, AnalogConfig, AnalogScalingMode, Comtrade, ComtradeParserBuilder, DataFormat,
    FormatRevision, LeapSecondStatus, SamplingRate, StatusChannel, StatusConfig, TimeQuality,
};

mod common;

use common::{assert_comtrades_eq, HOUR, MINUTE, SAMPLE_COMTRADE_DIR};

#[test]
fn it_correctly_parses_sample_2013_files_with_binary16_data() {
    let dir = Path::new(SAMPLE_COMTRADE_DIR);
    let cfg_path = dir.join("sample_2013_bin.cfg");
    let dat_path = dir.join("sample_2013_bin.dat");

    println!("CFG path: {:?}, DAT path: {:?}", cfg_path, dat_path);

    let cfg_file = BufReader::new(File::open(cfg_path).expect("unable to find sample cfg file"));
    let dat_file = BufReader::new(File::open(dat_path).expect("unable to find sample dat file"));

    let record = ComtradeParserBuilder::new()
        .cfg_file(cfg_file)
        .dat_file(dat_file)
        .build()
        .parse()
        .expect("unable to parse COMTRADE files");

    let expected_sample_rate = 15360.0;

    let expected = Comtrade {
        declared_total_channels: 20,
        station_name: "station".to_string(),
        recording_device_id: "equipment".to_string(),
        revision: FormatRevision::Revision2013,
        line_frequency: 60.0,
        sampling_rates: vec![SamplingRate {
            rate_hz: expected_sample_rate,
            end_sample_number: 5,
        }],
        start_time: NaiveDate::from_ymd(2017, 01, 07).and_hms_micro(15, 35, 41, 958_268),
        trigger_time: NaiveDate::from_ymd(2017, 01, 07).and_hms_micro(15, 35, 41, 958_333),
        data_format: DataFormat::Binary16,
        timestamp_multiplication_factor: 1.0,
        time_offset: Some(FixedOffset::west(5 * HOUR + 30 * MINUTE)),
        local_offset: Some(FixedOffset::west(5 * HOUR + 30 * MINUTE)),
        time_quality: Some(TimeQuality::ClockUnlocked(1)),
        leap_second_status: Some(LeapSecondStatus::NoCapability),

        sample_numbers: (1..=5).collect(),
        timestamps: (0..5).map(|i| i as f64 / expected_sample_rate).collect(),

        analog_channels: vec![
            AnalogChannel {
                config: AnalogConfig {
                    index: NonZeroUsize::new(1).unwrap(),
                    name: "VA".to_string(),
                    phase: "A".to_string(),
                    circuit_component_being_monitored: "obj".to_string(),
                    units: "kV".to_string(),
                    min_value: -32767.0,
                    max_value: 32767.0,
                    multiplier: 0.000361849,
                    offset_adder: 0.0,
                    skew: 0.0,
                    primary_factor: 120.0,
                    secondary_factor: 1.0,
                    scaling_mode: AnalogScalingMode::Primary,
                },
                data: vec![
                    -9.038625717163086,
                    -8.890992164611816,
                    -8.703554153442383,
                    -8.476312637329102,
                    -8.246539115905762,
                ],
            },
            AnalogChannel {
                config: AnalogConfig {
                    index: NonZeroUsize::new(2).unwrap(),
                    name: "VB".to_string(),
                    phase: "B".to_string(),
                    circuit_component_being_monitored: "obj".to_string(),
                    units: "kV".to_string(),
                    min_value: -32767.0,
                    max_value: 32767.0,
                    multiplier: 0.000365758,
                    offset_adder: 0.0,
                    skew: 0.0,
                    primary_factor: 120.0,
                    secondary_factor: 1.0,
                    scaling_mode: AnalogScalingMode::Primary,
                },
                data: vec![
                    -1.4282850027084350,
                    -1.6440821886062622,
                    -1.8617081642150880,
                    -2.0796999931335450,
                    -2.2852559089660645,
                ],
            },
            AnalogChannel {
                config: AnalogConfig {
                    index: NonZeroUsize::new(3).unwrap(),
                    name: "VC".to_string(),
                    phase: "C".to_string(),
                    circuit_component_being_monitored: "obj".to_string(),
                    units: "kV".to_string(),
                    min_value: -32767.0,
                    max_value: 32767.0,
                    multiplier: 0.000371569,
                    offset_adder: 0.0,
                    skew: 0.0,
                    primary_factor: 120.0,
                    secondary_factor: 1.0,
                    scaling_mode: AnalogScalingMode::Primary,
                },
                data: vec![
                    10.302122116088867,
                    10.383867263793945,
                    10.435143470764160,
                    10.448148727416992,
                    10.444433212280273,
                ],
            },
            AnalogChannel {
                config: AnalogConfig {
                    index: NonZeroUsize::new(4).unwrap(),
                    name: "VN".to_string(),
                    phase: "N".to_string(),
                    circuit_component_being_monitored: "obj".to_string(),
                    units: "kV".to_string(),
                    min_value: -32767.0,
                    max_value: 32767.0,
                    multiplier: 0.000016493,
                    offset_adder: 0.0,
                    skew: 0.0,
                    primary_factor: 60.0,
                    secondary_factor: 1.0,
                    scaling_mode: AnalogScalingMode::Primary,
                },
                data: vec![
                    0.20307831466197968,
                    0.19676148891448975,
                    0.19100543856620789,
                    0.18787176907062530,
                    0.18261049687862396,
                ],
            },
        ],

        status_channels: vec![
            StatusChannel {
                config: StatusConfig {
                    index: NonZeroUsize::new(1).unwrap(),
                    name: "ST_1".to_string(),
                    phase: "".to_string(),
                    circuit_component_being_monitored: "".to_string(),
                    normal_status_value: 0,
                },
                data: vec![0, 0, 0, 0, 0],
            },
            StatusChannel {
                config: StatusConfig {
                    index: NonZeroUsize::new(2).unwrap(),
                    name: "ST_2".to_string(),
                    phase: "".to_string(),
                    circuit_component_being_monitored: "".to_string(),
                    normal_status_value: 0,
                },
                data: vec![0, 0, 0, 0, 0],
            },
            StatusChannel {
                config: StatusConfig {
                    index: NonZeroUsize::new(3).unwrap(),
                    name: "ST_3".to_string(),
                    phase: "".to_string(),
                    circuit_component_being_monitored: "".to_string(),
                    normal_status_value: 0,
                },
                data: vec![0, 0, 0, 0, 0],
            },
            StatusChannel {
                config: StatusConfig {
                    index: NonZeroUsize::new(4).unwrap(),
                    name: "ST_4".to_string(),
                    phase: "".to_string(),
                    circuit_component_being_monitored: "".to_string(),
                    normal_status_value: 0,
                },
                data: vec![0, 0, 0, 0, 0],
            },
            StatusChannel {
                config: StatusConfig {
                    index: NonZeroUsize::new(5).unwrap(),
                    name: "ST_5".to_string(),
                    phase: "".to_string(),
                    circuit_component_being_monitored: "".to_string(),
                    normal_status_value: 0,
                },
                data: vec![0, 0, 0, 0, 0],
            },
            StatusChannel {
                config: StatusConfig {
                    index: NonZeroUsize::new(6).unwrap(),
                    name: "ST_6".to_string(),
                    phase: "".to_string(),
                    circuit_component_being_monitored: "".to_string(),
                    normal_status_value: 0,
                },
                data: vec![0, 0, 0, 0, 0],
            },
            StatusChannel {
                config: StatusConfig {
                    index: NonZeroUsize::new(7).unwrap(),
                    name: "ST_7".to_string(),
                    phase: "".to_string(),
                    circuit_component_being_monitored: "".to_string(),
                    normal_status_value: 0,
                },
                data: vec![0, 0, 0, 0, 0],
            },
            StatusChannel {
                config: StatusConfig {
                    index: NonZeroUsize::new(8).unwrap(),
                    name: "ST_8".to_string(),
                    phase: "".to_string(),
                    circuit_component_being_monitored: "".to_string(),
                    normal_status_value: 0,
                },
                data: vec![0, 0, 0, 0, 0],
            },
            StatusChannel {
                config: StatusConfig {
                    index: NonZeroUsize::new(9).unwrap(),
                    name: "ST_9".to_string(),
                    phase: "".to_string(),
                    circuit_component_being_monitored: "".to_string(),
                    normal_status_value: 0,
                },
                data: vec![0, 0, 0, 0, 0],
            },
            StatusChannel {
                config: StatusConfig {
                    index: NonZeroUsize::new(10).unwrap(),
                    name: "ST_10".to_string(),
                    phase: "".to_string(),
                    circuit_component_being_monitored: "".to_string(),
                    normal_status_value: 0,
                },
                data: vec![0, 0, 0, 0, 0],
            },
            StatusChannel {
                config: StatusConfig {
                    index: NonZeroUsize::new(11).unwrap(),
                    name: "ST_11".to_string(),
                    phase: "".to_string(),
                    circuit_component_being_monitored: "".to_string(),
                    normal_status_value: 0,
                },
                data: vec![0, 0, 0, 0, 0],
            },
            StatusChannel {
                config: StatusConfig {
                    index: NonZeroUsize::new(12).unwrap(),
                    name: "ST_12".to_string(),
                    phase: "".to_string(),
                    circuit_component_being_monitored: "".to_string(),
                    normal_status_value: 0,
                },
                data: vec![0, 0, 0, 0, 0],
            },
            StatusChannel {
                config: StatusConfig {
                    index: NonZeroUsize::new(13).unwrap(),
                    name: "ST_13".to_string(),
                    phase: "".to_string(),
                    circuit_component_being_monitored: "".to_string(),
                    normal_status_value: 0,
                },
                data: vec![0, 0, 0, 0, 0],
            },
            StatusChannel {
                config: StatusConfig {
                    index: NonZeroUsize::new(14).unwrap(),
                    name: "ST_14".to_string(),
                    phase: "".to_string(),
                    circuit_component_being_monitored: "".to_string(),
                    normal_status_value: 0,
                },
                data: vec![0, 0, 0, 0, 0],
            },
            StatusChannel {
                config: StatusConfig {
                    index: NonZeroUsize::new(15).unwrap(),
                    name: "ST_15".to_string(),
                    phase: "".to_string(),
                    circuit_component_being_monitored: "".to_string(),
                    normal_status_value: 0,
                },
                data: vec![0, 0, 0, 0, 0],
            },
            StatusChannel {
                config: StatusConfig {
                    index: NonZeroUsize::new(16).unwrap(),
                    name: "ST_16".to_string(),
                    phase: "".to_string(),
                    circuit_component_being_monitored: "".to_string(),
                    normal_status_value: 0,
                },
                data: vec![0, 0, 0, 0, 0],
            },
        ],
    };

    assert_comtrades_eq(&expected, &record);
}
