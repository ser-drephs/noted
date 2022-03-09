use std::{
    fmt::{self, Display, Formatter},
    io::{Error, ErrorKind},
    str,
};

/// File Rolling to determinate the note file rolling cycle.
#[derive(Debug, PartialEq, Clone)]
pub enum FileRolling {
    /// new file every day
    Daily,
    /// new file every week
    Week,
    /// new file every month
    Month,
    /// new file every year
    Year,
    /// never create a new file
    Never,
}

impl str::FromStr for FileRolling {
    type Err = Error;
    fn from_str(input: &str) -> Result<FileRolling, Self::Err> {
        let i: &str = &input.to_lowercase();
        log::debug!("Try parsing '{}' to file rolling", &i);
        match i {
            "daily" => Ok(FileRolling::Daily),
            "week" => Ok(FileRolling::Week),
            "month" => Ok(FileRolling::Month),
            "year" => Ok(FileRolling::Year),
            "never" => Ok(FileRolling::Never),
            _ => Err({
                let fmt = format!("unable to parse file rolling from '{}'", input);
                Error::new(ErrorKind::InvalidInput, fmt)
            }),
        }
    }
}

#[cfg(not(tarpaulin_include))]
impl Display for FileRolling {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::file_rolling::FileRolling;

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
}
