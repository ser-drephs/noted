use std::{
    io::{Error, ErrorKind},
    path::{Path, PathBuf}, env,
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
