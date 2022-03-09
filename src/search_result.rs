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

#[cfg(test)]
mod tests {

    use crate::search_result::SearchResult;

    macro_rules! test_format {
        ($name:ident, $($a:expr, $e:expr),+) => {
            #[test]
            fn $name() {
                $({
                let table = SearchResult::to_table($a);
                assert_eq!($e, table[1]);
                })*
            }
        };
    }

    test_format!(
        when_format_short_filename_then_no_dots_used,
        vec![("/temp/long/filename.md".to_string(), 1, "test".to_string())],
        "/temp/long/filename.md         | 1    | test"
    );

    test_format!(
        when_format_long_filename_then_dots_are_used,
        vec![(
            "/temp/long/and/longer/or/evenlonger/filename.md".to_string(),
            1,
            "test".to_string()
        )],
        "...r/or/evenlonger/filename.md | 1    | test"
    );

    test_format!(
        when_format_line_number_four_digits_then_table_size_respected,
        vec![(
            "/temp/long/and/longer/or/evenlonger/filename.md".to_string(),
            9999,
            "test".to_string(),
        )],
        "...r/or/evenlonger/filename.md | 9999 | test"
    );

    test_format!(
        when_format_line_number_five_digits_then_table_size_expanded,
        vec![(
            "/temp/long/and/longer/or/evenlonger/filename.md".to_string(),
            99999,
            "test".to_string(),
        )],
        "...r/or/evenlonger/filename.md | 99999 | test"
    );

    test_format!(
        when_format_long_line_then_dots_are_used,
        vec![(
            "/temp/long/and/longer/or/evenlonger/filename.md".to_string(),
            9999,
            "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.".to_string()
        )],
        "...r/or/evenlonger/filename.md | 9999 | Lorem Ipsum is simply dummy text of the pr..."
    );
}
