#[macro_export]
macro_rules! vec_of_strings {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

#[macro_export]
macro_rules! str {
    ($x:expr, PathBuf) => {
        $x.to_str().unwrap().to_string()
    };
    ($x:expr) => {
        $x.to_string()
    };
}

#[macro_export]
macro_rules! assert_file_exists {
    ($($x:expr),*) => {
        $({
            for _i in [1..5]{
                if $x.exists(){
                    break
                } else {
                    let mut child = std::process::Command::new("sleep").arg("1").spawn().unwrap();
                    let _result = child.wait().unwrap();
                }
            }
        })*
    };
}

#[macro_export]
macro_rules! safe_file_create{
    ($($x:expr),*) => {
        $({
            std::fs::File::create($x).unwrap();
            // wait_file_created::robust_wait_read($x).unwrap();
        })*
    };
}

#[cfg(windows)]
pub const LINE_ENDING: &str = "\r\n";
#[cfg(not(windows))]
pub const LINE_ENDING: &str = "\n";

pub const NOTES_FILE_NAME: &str = "notes.md";

pub mod cli;
pub mod command;
pub mod configuration;
pub mod file_rolling;
pub mod markdown;
pub mod note_file;
pub mod note_template;
pub mod note;
pub mod search_result;

/// Represents the formated note
pub struct FormatedNote {
    content: String,
}

/// Represents search arguments
#[derive(Debug, PartialEq)]
pub struct SearchArguments {
    pub regex: String,
    pub tags_only: bool,
    pub file_regex: Option<String>,
}

impl Default for SearchArguments {
    fn default() -> Self {
        SearchArguments {
            regex: str!(""),
            file_regex: None,
            tags_only: false,
        }
    }
}