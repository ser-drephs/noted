use std::{
    fmt::{self, Display, Formatter},
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

#[cfg(test)]
mod tests {
    use crate::{cli::Cli, command::Command};

    macro_rules! test_command {
        ($name:ident, $($s:expr, $o:expr),+) => {
            #[test]
            fn $name() {
                $({
                    let res = Cli::parse($s.iter()).unwrap();
                    assert_eq!(
                    $o,
                    res.command
                )})*
            }
        };
    }

    test_command!(
        when_command_note_is_invoke_then_take_note,
        ["noted", "take some note"],
        Command::Note {
            open_after_write: false,
            note: str!("take some note"),
            tags: Vec::new()
        }
    );

    test_command!(
        when_command_take_note_with_flag_open_short_then_take_note_and_open_after,
        ["noted", "take some note", "-o"],
        Command::Note {
            open_after_write: true,
            note: str!("take some note"),
            tags: Vec::new()
        }
    );

    test_command!(
        when_commdn_take_note_with_single_tag_then_take_note_with_tag,
        ["noted", "take some note", "Tag1"],
        Command::Note {
            open_after_write: false,
            note: str!("take some note"),
            tags: [str!("Tag1")].to_vec()
        }
    );

    test_command!(
        when_command_take_note_with_multiple_tags_then_take_note_with_multiple_tags,
        ["noted", "take some note", "Tag1", "Tag2"],
        Command::Note {
            open_after_write: false,
            note: str!("take some note"),
            tags: [str!("Tag1"), str!("Tag2")].to_vec()
        }
    );

    test_command!(
        when_command_take_note_with_multiple_tags_and_open_then_take_note_with_tags_and_open_after,
        ["noted", "take some note", "Tag1", "Tag2", "-o"],
        Command::Note {
            open_after_write: true,
            note: str!("take some note"),
            tags: [str!("Tag1"), str!("Tag2")].to_vec()
        }
    );

    test_command!(
        when_command_take_note_with_multiple_tags_and_open_are_mixedthen_take_note_with_tags_and_open_after,
        ["noted", "take some note", "Tag1", "-o", "Tag2", "Tag3"],
        Command::Note {
            open_after_write: true,
            note: str!("take some note"),
            tags: [str!("Tag1"), str!("Tag2"), str!("Tag3")].to_vec()
        }
    );

    #[test]
    fn when_command_create_has_no_args_then_help_is_shown_about_required_arguments() {
        let err = Cli::parse(["noted", "create"].iter()).unwrap_err();
        assert!(err
            .message
            .contains("The following required arguments were not provided"));
        assert!(err.message.contains("filename"));
    }

    test_command!(
        when_command_create_with_filename_then_create_file_with_specific_name,
        ["noted", "create", "test"],
        Command::Create {
            filename: str!("test")
        }
    );

    test_command!(
        when_command_create_invoked_using_alias_c_then_create_note,
        ["noted", "c", "test"],
        Command::Create {
            filename: str!("test")
        }
    );

    test_command!(
        when_command_create_invoked_using_alias_new_then_create_note,
        ["noted", "new", "test"],
        Command::Create {
            filename: str!("test")
        }
    );

    test_command!(
        when_command_create_invoked_using_alias_n_then_create_note,
        ["noted", "n", "test"],
        Command::Create {
            filename: str!("test")
        }
    );

    test_command!(
        when_command_open_without_filename_then_open_current_note_file,
        ["noted", "open"],
        Command::Open { filename: None }
    );

    test_command!(
        when_commdn_open_with_filename_then_open_note_file_with_filename,
        ["noted", "open", "file"],
        Command::Open {
            filename: Some(str!("file"))
        }
    );

    test_command!(
        when_command_open_invoked_using_alias_o_then_open_current_note_file,
        ["noted", "open"],
        Command::Open { filename: None }
    );

    test_command!(
        when_command_open_invoked_using_alias_edit_then_open_note_file_with_filename,
        ["noted", "edit"],
        Command::Open { filename: None }
    );

    test_command!(
        when_command_open_invoked_using_alias_e_then_open_current_note_file,
        ["noted", "e"],
        Command::Open { filename: None }
    );

    test_command!(
        when_command_open_invoked_using_alias_view_then_open_current_note_file,
        ["noted", "view"],
        Command::Open { filename: None }
    );

    test_command!(
        when_command_search_invoked_with_pattern_then_search_for_pattern,
        ["noted", "search", "xyz*"],
        Command::Search {
            tag: false,
            pattern: str!("xyz*"),
            file_pattern: None,
            output_to_file: false
        }
    );

    test_command!(
        when_command_search_invoked_with_pattern_and_flag_tag_then_search_for_pattern_only_for_tags,
        ["noted", "search", "--tag", "xyz*"],
        Command::Search {
            tag: true,
            pattern: str!("xyz*"),
            file_pattern: None,
            output_to_file: false
        }
    );

    test_command!(
        when_command_search_ionvoked_with_pattern_flag_tag_and_filepattern_then_search_for_pattern_only_for_tags_with_filefilter,
        ["noted", "search", "--tag", "xyz*", "*samplefile*"],
        Command::Search {
            tag: true,
            pattern: str!("xyz*"),
            file_pattern: Some(str!("*samplefile*")),
            output_to_file: false
        }
    );

    test_command!(
        when_command_search_invoked_with_pattern_short_flag_tag_then_search_for_pattern_only_for_tags,
        ["noted", "search", "-t", "xyz*"],
        Command::Search {
            tag: true,
            pattern: str!("xyz*"),
            file_pattern: None,
            output_to_file: false
        }
    );

    test_command!(
        when_command_search_invoked_using_alias_s_then_search,
        ["noted", "s", "-t", "xyz*"],
        Command::Search {
            tag: true,
            pattern: str!("xyz*"),
            file_pattern: None,
            output_to_file: false
        }
    );

    test_command!(
        when_command_search_invoked_using_alias_grep_then_search,
        ["noted", "grep", "xyz*"],
        Command::Search {
            tag: false,
            pattern: str!("xyz*"),
            file_pattern: None,
            output_to_file: false
        }
    );

    test_command!(
        when_command_search_invoked_using_alias_find_then_search,
        ["noted", "find", "xyz*"],
        Command::Search {
            tag: false,
            pattern: str!("xyz*"),
            file_pattern: None,
            output_to_file: false
        }
    );

    test_command!(
        when_command_search_invoked_using_alias_f_then_search,
        ["noted", "f", "xyz*"],
        Command::Search {
            tag: false,
            pattern: str!("xyz*"),
            file_pattern: None,
            output_to_file: false
        }
    );

    test_command!(
        when_command_config_invoked_then_open_config_file,
        ["noted", "config"],
        Command::Config
    );

    // Known Limitaion: Currently it is not possible (at least I didn't find anythin suitable) to disable InferSubcommands in clap.
    // Therefore this bug will persist.
    //     use {Cli, Command};

    //     test_command!(
    //         command_create_like_defaults_to_note,
    //         ["noted", "creat undefined"],
    //         Command::Note {
    //             open_after_write: false,
    //             note: "creat undefined".to_string(),
    //             tags: Vec::new()
    //         }
    //     );

    //     test_command!(
    //         command_open_like_defaults_to_note,
    //         ["noted", "ope undefined"],
    //         Command::Note {
    //             open_after_write: false,
    //             note: "ope undefined".to_string(),
    //             tags: Vec::new()
    //         }
    //     );

    //     test_command!(
    //         command_search_like_defaults_to_note,
    //         ["noted", "searc undefined"],
    //         Command::Note {
    //             open_after_write: false,
    //             note: "searc undefined".to_string(),
    //             tags: Vec::new()
    //         }
    //     );

    //     test_command!(
    //         command_config_like_defaults_to_note,
    //         ["noted", "conf undefined"],
    //         Command::Note {
    //             open_after_write: false,
    //             note: "conf undefined".to_string(),
    //             tags: Vec::new()
    //         }
    //     );
}
