use std::{
    fmt::{Display, Formatter, self},
    io::Error,
    path::PathBuf,
};

use crate::{
    configuration::Configuration, markdown::Markdown, note_file::NoteFile,
    search_result::SearchResult, SearchArguments,
};

/// represents the command to be executed
#[derive(Debug, PartialEq)]
pub enum Command {
    Default,
    Note {
        open_after_write: bool,
        note: String,
        tags: Vec<String>,
    },
    Create {
        filename: String,
    },
    Open {
        filename: Option<String>,
    },
    Search {
        tag: bool,
        pattern: String,
        file_pattern: Option<String>,
        /// Only used for tests!
        output_to_file: bool,
    },
    Config,
}

#[cfg(not(tarpaulin_include))]
impl Display for Command {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Command {
    pub fn invoke(&self, configuration: Option<Configuration>) -> Result<Option<PathBuf>, Error> {
        use Command::*;

        // create or read configuration
        let configuration = if let Some(config) = configuration {
            config
        } else {
            Configuration::new()
        };

        log::debug!("Invoke command: {}", self);

        match self {
            Default => todo!(),
            Note {
                open_after_write,
                note,
                tags,
            } => {
                // create note entry
                let note = crate::note::Note {
                    content: note.to_string(),
                    tags: tags.to_vec(),
                }
                .format(&configuration.note_template);
                // write to markdown
                match Markdown::from(NoteFile::target(&configuration)).write(&note) {
                    Ok(filepath) => Ok(if open_after_write.to_owned() {
                        Some(filepath)
                    } else {
                        None
                    }),
                    Err(err) => Err(err),
                }
            }
            Create { filename } => {
                // create note entry
                let note = crate::note::Note::default().format(&configuration.note_template);
                match Markdown::from(&NoteFile::custom_target(filename, &configuration))
                    .write(&note)
                {
                    Ok(file) => Ok(Some(file)),
                    Err(err) => Err(err),
                }
            }
            Open { filename } => {
                // if additional string is provided in arguments try to find a file with the filename in note directory
                let file = if let Some(pattern) = filename {
                    NoteFile::first_target_by_pattern(
                        pattern,
                        &PathBuf::from(&configuration.note_directory),
                    )
                    .unwrap()
                } else {
                    // othwise open current file
                    NoteFile::target(&configuration)
                };
                Ok(Some(file))
            }
            Search {
                tag,
                pattern,
                file_pattern,
                output_to_file,
            } => {
                match Markdown::search(
                    SearchArguments {
                        regex: pattern.to_owned(),
                        tags_only: tag.to_owned(),
                        file_regex: file_pattern.to_owned(),
                    },
                    &configuration,
                ) {
                    Ok(res) => {
                        let result = SearchResult::to_table(res);

                        if output_to_file.to_owned() {
                            SearchResult::write(&result).unwrap();
                        }
                        Ok(None)
                    }
                    Err(err) => Err(err),
                }
            }
            Config => {
                // open config in default editor
                Ok(Some(Configuration::file_path()))
            }
        }
    }
}
