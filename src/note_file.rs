use std::{
    env,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

use crate::{configuration::Configuration, file_rolling::FileRolling, NOTES_FILE_NAME};

/// represents a note file
#[derive(Debug, PartialEq)]
pub struct NoteFile {
    pub file: String,
}

impl From<&FileRolling> for NoteFile {
    fn from(val: &FileRolling) -> Self {
        use FileRolling::*;
        let now = chrono::Local::now();
        log::debug!("Note file name based on file rolling: {}", val);
        NoteFile {
            file: match val {
                Daily => format!("{}.md", &now.format("%Y-%m-%d")),
                Week => format!("{}.md", &now.format("%Y-%W")),
                Month => format!("{}.md", &now.format("%Y-%m")),
                Year => format!("{}.md", &now.format("%Y")),
                Never => NOTES_FILE_NAME.to_string(),
            },
        }
    }
}

impl From<PathBuf> for NoteFile {
    fn from(path: PathBuf) -> Self {
        NoteFile {
            file: str!(path, PathBuf),
        }
    }
}

impl NoteFile {
    /// search for note files in provided base_dir for pattern by regex
    pub fn target_by_pattern(regex: &str, base_dir: &Path) -> Result<Vec<PathBuf>, Error> {
        if !regex.is_empty() {
            let options = glob::MatchOptions {
                case_sensitive: false,
                ..Default::default()
            };
            let regex_path = base_dir.join(regex);
            log::debug!("Search for {:?}.", &regex_path);

            match glob::glob_with(regex_path.to_str().unwrap(), options) {
                Ok(paths) => {
                    let mut filter_result: Vec<_> = paths
                        .filter_map(Result::ok)
                        // .map(|f| base_dir.join(f))
                        .collect();
                    filter_result.sort_unstable();

                    log::debug!(
                        "Found {} files that match pattern {}: {:?}",
                        filter_result.len(),
                        regex_path.to_str().unwrap(),
                        filter_result
                    );

                    if filter_result.is_empty() {
                        let err_msg = "Result is empty.";
                        log::error!("{}", err_msg);
                        Err(Error::new(ErrorKind::NotFound, err_msg))
                    } else {
                        Ok(filter_result)
                    }
                }
                Err(err) => Err(Error::new(ErrorKind::Other, err)),
            }
        } else {
            let err_msg = "Pattern is empty";
            log::error!("{}", err_msg);
            Err(Error::new(ErrorKind::Other, err_msg))
        }
    }

    pub fn first_target_by_pattern(regex: &str, base_dir: &Path) -> Result<PathBuf, Error> {
        match NoteFile::target_by_pattern(regex, base_dir) {
            Ok(files) => Ok(files.get(0).unwrap().clone()),
            Err(err) => Err(err),
        }
    }

    /// get current target note file based on configuration
    pub fn target(configuration: &Configuration) -> PathBuf {
        let try_repo_specific = if configuration.use_repository_specific {
            let cur_dir = env::current_dir().unwrap();
            log::debug!(
                "Configured repository specific note file. Searching for note file in: {}",
                str!(cur_dir, PathBuf)
            );
            match &git2::Repository::discover(cur_dir) {
                Ok(repo) => {
                    log::debug!("Repository found. Using repository specific note file.");
                    Some(path_clean::PathClean::clean(
                        &repo.path().join("..").join(NOTES_FILE_NAME),
                    ))
                }
                Err(_) => {
                    log::info!("Repository not found.");
                    None
                }
            }
        } else {
            None
        };
        if let Some(repo_specific) = try_repo_specific {
            repo_specific
        } else {
            let notefile = NoteFile::from(&configuration.file_rolling);
            PathBuf::from(&configuration.note_directory).join(notefile.file)
        }
    }

    /// create a custom target file
    pub fn custom_target(filename: &str, configuration: &Configuration) -> PathBuf {
        if filename.is_empty() {
            NoteFile::target(configuration)
        } else {
            let new_filename = if !filename.ends_with(".md") {
                format!("{}.md", filename)
            } else {
                filename.to_string()
            };
            PathBuf::from(&configuration.note_directory).join(new_filename)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{file_rolling::FileRolling, NOTES_FILE_NAME, configuration::Configuration};

    use super::NoteFile;

    #[test]
    fn when_search_takes_invalid_pattern_then_error_is_raised() {
        match NoteFile::target_by_pattern("[", tempfile::tempdir().unwrap().path()) {
            Ok(_) => panic!("should not be ok"),
            Err(err) => {
                assert_eq!(std::io::ErrorKind::Other, err.kind());
            }
        }
    }

    macro_rules! from_tests {
        ($name:ident, $($s:expr, $o:expr),+) => {
            #[test]
            fn $name() {
                let now = chrono::Local::now();
                $({
                    let file_name = format!("{}.md", &now.format($o));
                    let res = NoteFile::from(&$s);
                    assert_eq!(
                    file_name,
                    res.file
                )})*
            }
        };
    }
    from_tests!(when_file_rolling_is_daily_then_daily_pattern_is_used, FileRolling::Daily, "%Y-%m-%d");
    from_tests!(when_file_rolling_is_month_then_monthly_pattern_is_used, FileRolling::Month, "%Y-%m");
    from_tests!(when_file_rolling_is_week_then_weekly_pattern_is_used, FileRolling::Week, "%Y-%W");
    from_tests!(when_file_rolling_is_year_then_yearly_pattern_is_used, FileRolling::Year, "%Y");
    from_tests!(when_file_rolling_is_never_then_fixed_name_is_used, FileRolling::Never, "notes");

    macro_rules! custom_target_tests {
    ($name:ident, $($a:expr, $e:expr),+) => {
        #[test]
        fn $name() {
            $({
                let configuration = Configuration{
                    file_rolling: FileRolling::Never,
                    ..Default::default()
                };
                let target = NoteFile::custom_target($a, &configuration);
                assert_eq!(std::path::PathBuf::from(configuration.note_directory).join($e), target);
            })*
        }
    };
}

    custom_target_tests!(when_custom_target_file_with_md_extension_is_used_then_md_not_appended, "test.md", "test.md");
    custom_target_tests!(when_custom_target_file_without_md_extension_is_used_then_md_is_appended, "test", "test.md");
    custom_target_tests!(
        when_custom_target_file_with_other_extension_is_used_then_md_is_appended,
        "test.ini",
        "test.ini.md"
    );
    custom_target_tests!(
        when_custom_target_file_without_any_name_is_used_then_name_from_config_is_used,
        "",
        NOTES_FILE_NAME
    );

    #[test]
    fn when_custom_target_repository_specific_is_set_not_inside_repository_then_note_directory_is_used() {
        let configuration = Configuration {
            use_repository_specific: true,
            file_rolling: FileRolling::Never,
            ..Default::default()
        };
        let target = NoteFile::custom_target("sample_not_inside_repo", &configuration);
        assert_eq!(
            std::path::PathBuf::from(configuration.note_directory)
                .join("sample_not_inside_repo.md"),
            target
        );
    }

    #[test]
    fn when_user_directory_target_with_file_rolling_month_then_file_is_created() {
        let configuration = Configuration {
            file_rolling: FileRolling::Month,
            ..Default::default()
        };

        let now = chrono::Local::now();
        let file_name = format!("{}.md", &now.format("%Y-%m"));
        // ACT
        let target = NoteFile::target(&configuration);
        // ASSERT
        assert!(str!(target, PathBuf).starts_with("/"));
        assert!(str!(target, PathBuf).ends_with(&file_name));
    }

    #[test]
    fn when_invalid_pattern_for_target_is_used_then_error_bubbles_up() {
        match NoteFile::target_by_pattern("[", tempfile::tempdir().unwrap().path()) {
            Ok(_) => panic!("should not be ok"),
            Err(err) => {
                assert_eq!(std::io::ErrorKind::Other, err.kind());
            }
        }
    }
}
