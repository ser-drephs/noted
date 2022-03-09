use crate::LINE_ENDING;
use std::{
    fs::OpenOptions,
    io::{Error, Write},
    path::PathBuf,
};

/// Represents search results
#[derive(Debug)]
pub struct SearchResult {}

impl SearchResult {
    pub fn to_table(matches: Vec<(String, u64, String)>) -> Vec<String> {
        log::debug!("Formating {} occurences to a table", &matches.len());
        let mut table = vec![SearchResult::fmt("File", "Line", "Content")];
        for lines in matches {
            // cut filename to 30 chars. replace beginning if needed.
            let filepath = if lines.0.len() >= 30 {
                format!("...{}", &lines.0[&lines.0.len() - 27..])
            } else {
                lines.0
            };
            // cut occurence to 45 chars. replace end if needed
            let content = if lines.2.len() >= 45 {
                format!("{}...", &lines.2[0..42])
            } else {
                lines.2
            };
            table.push(SearchResult::fmt(&filepath, &str!(lines.1), &content));
        }
        for x in &table {
            println!("{}", x);
        }
        table
    }

    /// Write search results to file insted of stdout.
    /// Only used internally for tests!
    #[cfg(not(tarpaulin_include))]
    pub fn write(result: &[String]) -> Result<PathBuf, Error> {
        let log = PathBuf::from("search_result.txt");
        let file_write_error = format!("Could not write file at {}", str!(log, PathBuf));
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .truncate(false)
            .open(&log)
            .expect(&file_write_error);
        match Write::write_all(&mut file, result.join(LINE_ENDING).as_bytes()) {
            Ok(_) => Ok(log),
            Err(err) => Err(err),
        }
    }

    /// formatting of the search results table
    fn fmt(first: &str, second: &str, third: &str) -> String {
        format!("{0: <30} | {1: <4} | {2: <45}", first, second, third)
            .trim_end()
            .to_string()
    }
}
