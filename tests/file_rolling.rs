use std::str::FromStr;

use noted::file_rolling::FileRolling;

macro_rules! test_parse {
        ($name:ident, $($a:expr, $e:expr),+) => {
            #[test]
            fn $name() {
                $({
                    let file_rolling = FileRolling::from_str($a).unwrap();
                    assert_eq!($e, file_rolling);
                })*
            }
        };
    }

test_parse!(
    when_daily_string_is_parsed_then_daily_file_rolling_is_returned,
    "Daily",
    FileRolling::Daily
);
test_parse!(
    when_week_string_is_parsed_then_week_file_rolling_is_returned,
    "Week",
    FileRolling::Week
);
test_parse!(
    when_month_string_is_parsed_then_month_file_rolling_is_returned,
    "Month",
    FileRolling::Month
);
test_parse!(
    when_year_string_is_parsed_then_year_file_rolling_is_returned,
    "Year",
    FileRolling::Year
);
test_parse!(
    when_never_string_is_parsed_then_never_file_rolling_is_returned,
    "Never",
    FileRolling::Never
);
test_parse!(
    when_week_lowercase_string_is_parsed_then_week_file_rolling_is_returned,
    "week",
    FileRolling::Week
);

#[test]
fn when_invalid_string_is_parsed_then_an_error_is_returned() {
    let file_rolling = FileRolling::from_str("Abc");
    assert!(file_rolling.is_err());
    assert_eq!(
        "unable to parse file rolling from 'Abc'",
        file_rolling.unwrap_err().to_string()
    );
}
