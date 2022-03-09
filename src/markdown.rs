use std::{
    fs::{File, OpenOptions},
    io::{Error, ErrorKind, Read, Write},
    path::PathBuf,
};

use grep::{
    regex::RegexMatcher,
    searcher::{sinks, Searcher},
};

use crate::{configuration::Configuration, note_file::NoteFile, FormatedNote, SearchArguments};

/// Represents the markdown file
// #[deprecated]
pub struct Markdown {
    notefile: NoteFile,
}

impl From<PathBuf> for Markdown {
    fn from(path: PathBuf) -> Self {
        Markdown {
            notefile: NoteFile::from(path),
        }
    }
}

impl From<&PathBuf> for Markdown {
    fn from(path: &PathBuf) -> Self {
        Markdown {
            notefile: NoteFile::from(path.clone()),
        }
    }
}

impl Markdown {
    /// Write the formated note to the markdown file
    pub fn write(self, note: &FormatedNote) -> Result<PathBuf, Error> {
        match OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .truncate(false)
            .open(&self.notefile.file)
        {
            Ok(mut file) => match Write::write_all(&mut file, note.content.as_bytes()) {
                Ok(_) => {
                    log::debug!("Appending to note file: {}", &self.notefile.file);
                    Ok(PathBuf::from(self.notefile.file))
                }
                Err(err) => {
                    log::error!(
                        "Could not write to note file at {}: {:?}",
                        &self.notefile.file,
                        &err
                    );
                    Err(err)
                }
            },
            Err(err) => {
                log::error!(
                    "Could not create or append to note file at {}: {:?}",
                    &self.notefile.file,
                    &err
                );
                Err(err)
            }
        }
    }

    /// Reads notes from the markdown file.
    pub fn search(
        arguments: SearchArguments,
        configuration: &Configuration,
    ) -> Result<Vec<(String, u64, String)>, Error> {
        log::info!("Search string: '{}'", arguments.regex);
        if arguments.regex.is_empty() {
            let err_msg = "Search string is empty";
            log::error!("{}", err_msg);

            Err(Error::new(ErrorKind::InvalidInput, err_msg))
        } else {
            // first find all files that match the file pattern!
            let files = NoteFile::target_by_pattern(
                if let Some(file_regex) = &arguments.file_regex {
                    log::info!("Limit files to: '{}'", file_regex);
                    file_regex
                } else {
                    "*"
                },
                &PathBuf::from(&configuration.note_directory),
            );
            // if no files where found or there is an error, don't continue
            if let Ok(files) = files {
                let search_string = if arguments.tags_only {
                    format!("#{}", arguments.regex)
                } else {
                    // Format limiter as defined in SearchResult::to_table
                    let outer_limits = (45 - arguments.regex.len()) / 2;
                    format!(".?{{0,{0}}}{1}.?{{0,{0}}}", outer_limits, arguments.regex)
                };
                log::debug!("Using the following RegEx: {:?}", &search_string);
                let matcher = RegexMatcher::new(&search_string).unwrap();
                let mut matches: Vec<(String, u64, String)> = vec![];
                files.into_iter().for_each(|file| {
                    log::debug!("Try reading file: {:?}", &file);
                    // in the found files vec search for the provided pattern.
                    let mut buffer = String::new();
                    let mut f = File::open(&file).unwrap();
                    Read::read_to_string(&mut f, &mut buffer).unwrap();
                    Searcher::new()
                        .search_slice(
                            &matcher,
                            buffer.as_bytes(),
                            sinks::UTF8(|lnum, line| {
                                let mymatch =
                                    grep::matcher::Matcher::find(&matcher, line.as_bytes())?
                                        .unwrap();
                                // append each found occurence to the matches vec with filename, line and occurence.
                                matches.push((
                                    str!(file, PathBuf),
                                    lnum,
                                    line[mymatch].to_string(),
                                ));
                                Ok(true)
                            }),
                        )
                        .unwrap();
                });
                log::debug!("Found {} occurences", &matches.len());
                Ok(matches)
            } else {
                Err(files.err().unwrap())
            }
        }
    }
}
