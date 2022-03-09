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
