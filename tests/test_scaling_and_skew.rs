use comtrade::ComtradeParserBuilder;
use std::io::Cursor;

fn assert_near(actual: Option<f64>, expected: Option<f64>) {
    match (actual, expected) {
        (Some(a), Some(b)) => {
            assert!((a - b).abs() < 1e-9, "Expected {:?} to be near {:?}", a, b);
        }
        (None, None) => {}
        _ => panic!("Mismatch: actual={:?}, expected={:?}", actual, expected),
    }
}

#[test]
fn test_scaling_and_skew_calculations() {
    let cfg_data = "TEST_STATION,TEST_DEVICE,2013\n\
                    4,4A,0D\n\
                    1,IA,,Line1,A,0.5,10.0,100.0,-32768,32767,2000.0,5.0,p\n\
                    2,IB,,Line1,A,1.0,0.0,-50.0,-32768,32767,1000.0,5.0,s\n\
                    3,IC,,Line1,A,1.0,0.0,0.0,-32768,32767,1000.0,0.0,s\n\
                    4,ID,,Line1,A,1.0,0.0,0.0,-32768,32767,0.0,5.0,p\n\
                    60\n\
                    1\n\
                    1000,2\n\
                    12/01/2011,05:55:30.750110\n\
                    12/01/2011,05:55:30.782610\n\
                    ASCII\n\
                    1\n\
                    -5h30,-5h30\n\
                    B,3";

    let dat_data = "1,0,100,200,300,400\n\
                    2,1000,150,250,350,450\n";

    let cfg_cursor = Cursor::new(cfg_data);
    let dat_cursor = Cursor::new(dat_data);

    let comtrade = ComtradeParserBuilder::new()
        .cfg_file(cfg_cursor)
        .dat_file(dat_cursor)
        .build()
        .parse()
        .expect("Failed to parse synthetic COMTRADE files");

    // Let's assert values and timestamps
    // Channel 1: p (Primary Scaling Mode)
    // raw values: 100, 150
    // multiplier: 0.5, adder: 10.0
    // primary_factor: 2000.0, secondary_factor: 5.0
    let ch1 = &comtrade.analog_channels[0];

    // raw value calculation: raw_value * multiplier + adder
    // 100 * 0.5 + 10.0 = 60.0
    // 150 * 0.5 + 10.0 = 85.0
    assert_near(ch1.value(0), Some(60.0));
    assert_near(ch1.value(1), Some(85.0));
    assert_near(ch1.value(2), None);

    // primary_value returns the value itself since scaling_mode = Primary
    assert_near(ch1.primary_value(0), Some(60.0));
    assert_near(ch1.primary_value(1), Some(85.0));
    assert_near(ch1.primary_value(2), None);

    // secondary_value: val * (secondary_factor / primary_factor)
    // 60.0 * (5.0 / 2000.0) = 0.15
    // 85.0 * (5.0 / 2000.0) = 0.2125
    assert_near(ch1.secondary_value(0), Some(0.15));
    assert_near(ch1.secondary_value(1), Some(0.2125));
    assert_near(ch1.secondary_value(2), None);

    // Channel 2: s (Secondary Scaling Mode)
    // raw values: 200, 250
    // multiplier: 1.0, adder: 0.0
    // primary_factor: 1000.0, secondary_factor: 5.0
    let ch2 = &comtrade.analog_channels[1];

    // 200 * 1.0 + 0.0 = 200.0
    // 250 * 1.0 + 0.0 = 250.0
    assert_near(ch2.value(0), Some(200.0));
    assert_near(ch2.value(1), Some(250.0));
    assert_near(ch2.value(2), None);

    // secondary_value returns the value itself since scaling_mode = Secondary
    assert_near(ch2.secondary_value(0), Some(200.0));
    assert_near(ch2.secondary_value(1), Some(250.0));
    assert_near(ch2.secondary_value(2), None);

    // primary_value: val * (primary_factor / secondary_factor)
    // 200.0 * (1000.0 / 5.0) = 40000.0
    // 250.0 * (1000.0 / 5.0) = 50000.0
    assert_near(ch2.primary_value(0), Some(40000.0));
    assert_near(ch2.primary_value(1), Some(50000.0));
    assert_near(ch2.primary_value(2), None);

    // Channel 3: s (Secondary Scaling Mode) but secondary_factor = 0.0
    let ch3 = &comtrade.analog_channels[2];
    assert_near(ch3.value(0), Some(300.0));
    assert_near(ch3.secondary_value(0), Some(300.0));
    assert_near(ch3.primary_value(0), Some(300.0)); // falls back to val

    // Channel 4: p (Primary Scaling Mode) but primary_factor = 0.0
    let ch4 = &comtrade.analog_channels[3];
    assert_near(ch4.value(0), Some(400.0));
    assert_near(ch4.primary_value(0), Some(400.0));
    assert_near(ch4.secondary_value(0), Some(400.0)); // falls back to val

    // Timestamps checking
    // base_timestamps are from the dat file (converted to seconds relative to start time):
    // dat row 1: 0 microseconds -> 0.0 seconds
    // dat row 2: 1000 microseconds -> 0.001 seconds
    assert_eq!(comtrade.timestamps, vec![0.0, 0.001]);

    // Channel 1: skew = 100.0 microseconds -> +0.0001 seconds
    assert_near(ch1.timestamp_at(0, &comtrade.timestamps), Some(0.0001));
    assert_near(ch1.timestamp_at(1, &comtrade.timestamps), Some(0.0011));
    assert_near(ch1.timestamp_at(2, &comtrade.timestamps), None);

    // Channel 2: skew = -50.0 microseconds -> -0.00005 seconds
    assert_near(ch2.timestamp_at(0, &comtrade.timestamps), Some(-0.00005));
    assert_near(ch2.timestamp_at(1, &comtrade.timestamps), Some(0.00095));
    assert_near(ch2.timestamp_at(2, &comtrade.timestamps), None);
}
