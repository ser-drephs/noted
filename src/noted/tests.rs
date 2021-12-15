#[cfg(test)]
mod file_rolling_tests {
    use std::str::FromStr;

    use crate::noted::FileRolling;

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

    test_parse!(daily, "Daily", FileRolling::Daily);
    test_parse!(week, "Week", FileRolling::Week);
    test_parse!(month, "Month", FileRolling::Month);
    test_parse!(year, "Year", FileRolling::Year);
    test_parse!(never, "Never", FileRolling::Never);
    test_parse!(case_insensitive, "week", FileRolling::Week);

    #[test]
    fn parse_error_test() {
        let file_rolling = FileRolling::from_str("Abc");
        assert!(file_rolling.is_err());
        assert_eq!(
            "unable to parse file rolling from 'Abc'",
            file_rolling.unwrap_err().to_string()
        );
    }
}

#[cfg(test)]
mod configuration_tests {
    use crate::noted::{Configuration, FileRolling, Initialize, NoteTemplate};
    use crate::{str, vec_of_strings};
    use indoc::indoc;
    use serial_test::serial;

    #[test]
    fn default_configuration() {
        let configuration = Configuration::default();
        assert_eq!(FileRolling::Daily, configuration.file_rolling);
        assert_eq!(
            Configuration::intial_note_directory(),
            configuration.note_directory
        );
        assert_eq!("%F %T", configuration.note_template.date_format);
        assert_eq!(
            indoc! {"
            %date_format%

            %note%

            %tags%"},
            configuration.note_template.template
        );
        assert_eq!(
            str!(NoteTemplate::initial_file_path(), PathBuf),
            configuration.template_file
        );
        assert!(!configuration.use_repository_specific);
    }

    #[test]
    #[serial]
    fn write_configuration() {
        // arrange
        let expected_config = vec_of_strings!(
            "NOTE_DIRECTORY=/home/vscode",
            "USE_REPOSITORY_SPECIFIC=false",
            "FILE_ROLLING=Daily",
            "DATE_FORMAT=%F %T",
            "NOTE_TEMPLATE=home/vscode/.config/noted/notedtemplate.md"
        );
        let expected_template = vec_of_strings!("%date_format%", "", "%note%", "", "%tags%");
        let _r = std::fs::remove_dir_all(Configuration::folder());
        assert!(!Configuration::folder().exists());
        // act
        Configuration::new();
        // assert
        let read_config_file = std::fs::File::open(Configuration::file_path()).unwrap();
        let raw_config: Vec<String> =
            std::io::BufRead::lines(std::io::BufReader::new(read_config_file))
                .map(|l| l.unwrap())
                .collect();

        for i in 0..raw_config.len() - 1 {
            assert_eq!(expected_config[i], raw_config[i]);
        }
        let read_template_file = std::fs::File::open(NoteTemplate::initial_file_path()).unwrap();
        let raw_template: Vec<String> =
            std::io::BufRead::lines(std::io::BufReader::new(read_template_file))
                .map(|l| l.unwrap())
                .collect();

        for i in 0..raw_template.len() - 1 {
            assert_eq!(expected_template[i], raw_template[i]);
        }
        let removed = std::fs::remove_dir_all(Configuration::folder());
        assert!(removed.is_ok());
        assert!(!Configuration::folder().exists());
    }

    #[test]
    #[serial]
    fn read_default() {
        // arrange
        let _r = std::fs::remove_dir_all(Configuration::folder());
        assert!(!Configuration::folder().exists());
        // act
        Configuration::new();
        // act
        let config = Configuration::new();
        // assert
        assert_eq!("/home/vscode".to_string(), config.note_directory);
        assert!(!config.use_repository_specific);
        assert_eq!(FileRolling::Daily, config.file_rolling);
        assert_eq!(
            "/home/vscode/.config/noted/notedtemplate.md",
            config.template_file
        );
        assert!(config.note_template.template.starts_with("%date_format%"));
        // clean up
        let removed = std::fs::remove_dir_all(Configuration::folder());
        assert!(removed.is_ok());
        assert!(!Configuration::folder().exists());
    }
}

#[cfg(test)]
mod markdown_tests {

    mod search_tests {
        use crate::noted::{SearchArguments, Configuration, Markdown};

        #[test]
        fn empty_search_arguments_invalidinput_error() {
            let search = SearchArguments::default();
            let configuration = Configuration::default();
            match Markdown::search(search, &configuration) {
                Ok(_) => panic!("should not be ok"),
                Err(err) => {
                    assert_eq!(std::io::ErrorKind::InvalidInput, err.kind());
                }
            }
        }

        #[test]
        fn files_by_pattern_not_found_error() {
            let search = SearchArguments {
                regex: "For Unit Test Only Will Not Be Found".to_string(),
                file_regex: Some("__\\".to_string()),
                ..Default::default()
            }; //::from(vec_of_strings!("bug", "_\\"));
            let configuration = Configuration::default();
            match Markdown::search(search, &configuration) {
                Ok(_) => panic!("should not be ok"),
                Err(err) => {
                    assert_eq!(std::io::ErrorKind::NotFound, err.kind());
                }
            }
        }
    }
}

#[cfg(test)]
mod note_tests {
    use crate::noted::Note;
    use crate::vec_of_strings;

    macro_rules! test_note {
        ($name:ident, $($a:expr, $e:expr),+) => {
            #[test]
            fn $name() {
                $({
                    let note = Note::from($a);
                    assert_eq!($e, note);
                })*
            }
        };
    }

    test_note!(
        from_empty_vec_should_be_default,
        Vec::new(),
        Note::default()
    );

    test_note!(
        from_one_argument_should_create_note_without_tags,
        ["Sample note"].to_vec(),
        Note {
            content: "Sample note".to_string(),
            ..Default::default()
        }
    );

    test_note!(
        from_two_arguments_should_create_note_with_tag,
        ["Sample note", "tag1"].to_vec(),
        Note {
            content: "Sample note".to_string(),
            tags: vec_of_strings!("tag1")
        }
    );

    test_note!(
        from_multiple_arguments_should_create_note_with_multiple_tags,
        ["Sample note", "tag1", "tag2", "tag3", "tag4"].to_vec(),
        Note {
            content: "Sample note".to_string(),
            tags: vec_of_strings!("tag1", "tag2", "tag3", "tag4")
        }
    );
}

#[cfg(test)]
mod search_result_tests {
    use crate::noted::SearchResult;
    use crate::str;

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
        format_short_filename_no_dots,
        vec![(str!("/temp/long/filename.md"), 1, str!("test"))],
        "/temp/long/filename.md         | 1    | test"
    );

    test_format!(
        format_long_filename_with_dots,
        vec![(
            str!("/temp/long/and/longer/or/evenlonger/filename.md"),
            1,
            str!("test")
        )],
        "...r/or/evenlonger/filename.md | 1    | test"
    );

    test_format!(
        format_line_number_four_digits,
        vec![(
            str!("/temp/long/and/longer/or/evenlonger/filename.md"),
            9999,
            str!("test"),
        )],
        "...r/or/evenlonger/filename.md | 9999 | test"
    );

    test_format!(
        format_line_number_five_digits,
        vec![(
            str!("/temp/long/and/longer/or/evenlonger/filename.md"),
            99999,
            str!("test"),
        )],
        "...r/or/evenlonger/filename.md | 99999 | test"
    );

    test_format!(
        format_long_line,
        vec![(
            str!("/temp/long/and/longer/or/evenlonger/filename.md"),
            9999,
            str!("Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.")
        )],
        "...r/or/evenlonger/filename.md | 9999 | Lorem Ipsum is simply dummy text of the pr...");
}

#[cfg(test)]
mod notefile_tests {

    mod search_tests{
        use crate::noted::NoteFile;

        #[test]
        fn invalid_pattern(){
            match NoteFile::target_by_pattern("[", &std::env::current_dir().unwrap()){
                Ok(_) => panic!("should not be ok"),
                Err(err) => {
                    assert_eq!(std::io::ErrorKind::Other, err.kind());
                },
            }
        }
    }

    mod from_tests {
        use crate::noted::{FileRolling, NoteFile};

        macro_rules! test_conversion {
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

        test_conversion!(daily, FileRolling::Daily, "%Y-%m-%d");
        test_conversion!(month, FileRolling::Month, "%Y-%m");
        test_conversion!(week, FileRolling::Week, "%Y-%W");
        test_conversion!(year, FileRolling::Year, "%Y");
        test_conversion!(never, FileRolling::Never, "notes");
    }

    mod target_tests {
        use crate::noted::{Configuration, NoteFile};
        use crate::str;

        #[test]
        fn user_directory_file_rolling_month() {
            let configuration = Configuration {
                file_rolling: crate::noted::FileRolling::Month,
                ..Default::default()
            };

            let now = chrono::Local::now();
            let file_name = format!("{}.md", &now.format("%Y-%m"));
            // ACT
            let target = NoteFile::target(&configuration);
            // ASSERT
            assert!(str!(target, PathBuf).starts_with("/home"));
            assert!(str!(target, PathBuf).ends_with(&file_name));
        }

        #[test]
        fn invalid_pattern_bubble_up(){
            match NoteFile::target_by_pattern("[", &std::env::current_dir().unwrap()){
                Ok(_) => panic!("should not be ok"),
                Err(err) => {
                    assert_eq!(std::io::ErrorKind::Other, err.kind());
                },
            }
        }
    }

    mod custom_target_tests {
        use crate::noted::{Configuration, FileRolling, NoteFile, NOTES_FILE_NAME};

        macro_rules! test_custom_target {
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

        test_custom_target!(file_with_md_extension, "test.md", "test.md");
        test_custom_target!(file_without_md_extension, "test", "test.md");
        test_custom_target!(file_with_other_extension, "test.ini", "test.ini.md");
        test_custom_target!(file_without_name_returns_from_config, "", NOTES_FILE_NAME);

        #[test]
        fn custom_target_repository_specific_returns_note_directory() {
            let configuration = Configuration {
                use_repository_specific: true,
                file_rolling: FileRolling::Never,
                ..Default::default()
            };
            let target = NoteFile::custom_target("sample_not_inside_repo", &configuration);
            assert_eq!(std::path::PathBuf::from(configuration.note_directory).join("sample_not_inside_repo.md"), target);
        }
    }
}

#[cfg(test)]
mod cli_tests {

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

    mod help_tests {
        use indoc::indoc;

        use crate::noted::Cli;

        #[test]
        fn no_args_prints_help() {
            let err = Cli::parse(["noted"].iter()).unwrap_err();
            let expected = indoc!(
                "Take notes using CLI

                USAGE:
                    noted [FLAGS] <note> [tag]...
                    noted <SUBCOMMAND>

                FLAGS:
                    -o               Open note file in default editor after writing
                    -d               Set the level of verbosity
                    -h, --help       Prints help information
                    -v, --version    Prints version information

                ARGS:
                    <note>      Note to take
                    <tag>...    Tags for note

                SUBCOMMANDS:
                    create    Create note file and open in default editor
                    open      Opens note file in default editor
                    search    Search for a specific string in notes
                    config    Open configuration in default editor
                    help      Prints this message or the help of the given subcommand(s)"
            );
            assert!(
                err.message.contains(expected),
                "Expected: {:?}, Got: {:?}",
                expected,
                err.message
            );
        }

        #[test]
        fn arg_help() {
            let err = Cli::parse(["noted", "help"].iter()).unwrap_err();
            let expected = indoc!(
                "Take notes using CLI

                USAGE:
                    noted [FLAGS] <note> [tag]...
                    noted <SUBCOMMAND>

                FLAGS:
                    -o               Open note file in default editor after writing
                    -d               Set the level of verbosity
                    -h, --help       Prints help information
                    -v, --version    Prints version information

                ARGS:
                    <note>      Note to take
                    <tag>...    Tags for note

                SUBCOMMANDS:
                    create    Create note file and open in default editor
                    open      Opens note file in default editor
                    search    Search for a specific string in notes
                    config    Open configuration in default editor
                    help      Prints this message or the help of the given subcommand(s)"
            );
            assert!(err.message.contains(expected));
        }

        #[test]
        fn arg_help_create() {
            let err = Cli::parse(["noted", "help", "create"].iter()).unwrap_err();
            let expected = indoc!(
                "Creates a new note file in the configured note directory and opens it in default editor.

                USAGE:
                    noted create [FLAGS] <filename>

                FLAGS:
                    -d            Set the level of verbosity
                    -h, --help    Prints help information

                ARGS:
                    <filename>    File name for created note file"
            );
            assert!(err.message.contains(expected));
        }

        #[test]
        fn arg_help_open() {
            let err = Cli::parse(["noted", "help", "open"].iter()).unwrap_err();
            let expected = indoc!(
                "Open the current note file in the default editor.

                Depending on the configuration the current note file may also be repository specific.
                If filename is provided, a note file matching the pattern will be searched in the configured note directory.

                USAGE:
                    noted open [FLAGS] [filename]

                FLAGS:
                    -d            Set the level of verbosity
                    -h, --help    Prints help information

                ARGS:
                    <filename>    Provide filename for note to open"
            );
            assert!(err.message.contains(expected));
        }

        #[test]
        fn arg_help_search() {
            let err = Cli::parse(["noted", "help", "search"].iter()).unwrap_err();
            let expected = indoc!(
                "Search for a specific string in notes using the provided RegEx pattern.

                USAGE:
                    noted search [FLAGS] <pattern> [file filter]

                FLAGS:
                    -d            Set the level of verbosity
                    -h, --help    Prints help information
                    -t, --tag     Search only for tags

                ARGS:
                    <pattern>        Search pattern
                    <file filter>    File filter pattern"
            );
            assert!(err.message.contains(expected));
        }

        #[test]
        fn arg_help_config() {
            let err = Cli::parse(["noted", "help", "config"].iter()).unwrap_err();
            let expected = indoc!(
                "Open configuration in default editor

                USAGE:
                    noted config [FLAGS]

                FLAGS:
                    -d            Set the level of verbosity
                    -h, --help    Prints help information"
            );
            assert!(err.message.contains(expected));
        }

        #[test]
        fn arg_help_short_flag() {
            let err = Cli::parse(["noted", "create", "-h"].iter()).unwrap_err();
            assert!(err.message.contains("Creates a new note file in the configured note directory and opens it in default editor."));
            assert!(err.message.contains("File name for created note file"));
        }

        #[test]
        fn arg_help_long_flag() {
            let err = Cli::parse(["noted", "create", "--help"].iter()).unwrap_err();
            assert!(err.message.contains("Creates a new note file in the configured note directory and opens it in default editor."));
            assert!(err.message.contains(
                "A note file with the provided name is created in the configured note directory."
            ));
        }
    }

    mod verbosity_test {
        use crate::noted::Cli;

        #[test]
        fn default() {
            let res = Cli::parse(["noted", "take some note"].iter()).unwrap();
            assert_eq!(log::LevelFilter::Warn, res.verbosity);
        }

        #[test]
        fn info() {
            let res = Cli::parse(["noted", "-d", "take some note"].iter()).unwrap();
            assert_eq!(log::LevelFilter::Info, res.verbosity);
        }

        #[test]
        fn debug() {
            let res = Cli::parse(["noted", "-dd", "take some note"].iter()).unwrap();
            assert_eq!(log::LevelFilter::Debug, res.verbosity);
        }

        #[test]
        fn trace() {
            let res = Cli::parse(["noted", "-ddd", "file"].iter()).unwrap();
            assert_eq!(log::LevelFilter::Trace, res.verbosity);
        }

        #[test]
        fn subcommand_default() {
            let res = Cli::parse(["noted", "create", "file"].iter()).unwrap();
            assert_eq!(log::LevelFilter::Warn, res.verbosity);
        }

        #[test]
        fn subcommand_info() {
            let res = Cli::parse(["noted", "create", "-d", "file"].iter()).unwrap();
            assert_eq!(log::LevelFilter::Info, res.verbosity);
        }

        #[test]
        fn subcommand_debug() {
            let res = Cli::parse(["noted", "create", "-dd", "file"].iter()).unwrap();
            assert_eq!(log::LevelFilter::Debug, res.verbosity);
        }

        #[test]
        fn subcommand_trace() {
            let res = Cli::parse(["noted", "create", "-ddd", "file"].iter()).unwrap();
            assert_eq!(log::LevelFilter::Trace, res.verbosity);
        }
    }

    mod note_test {
        use crate::noted::{Cli, Command};
        use crate::str;

        test_command!(
            take_note,
            ["noted", "take some note"],
            Command::Note {
                open_after_write: false,
                note: str!("take some note"),
                tags: Vec::new()
            }
        );

        test_command!(
            take_note_open_short,
            ["noted", "take some note", "-o"],
            Command::Note {
                open_after_write: true,
                note: str!("take some note"),
                tags: Vec::new()
            }
        );

        test_command!(
            take_note_with_single_tag,
            ["noted", "take some note", "Tag1"],
            Command::Note {
                open_after_write: false,
                note: str!("take some note"),
                tags: [str!("Tag1")].to_vec()
            }
        );

        test_command!(
            take_note_with_multiple_tags,
            ["noted", "take some note", "Tag1", "Tag2"],
            Command::Note {
                open_after_write: false,
                note: str!("take some note"),
                tags: [str!("Tag1"), str!("Tag2")].to_vec()
            }
        );

        test_command!(
            take_note_with_multiple_tags_open,
            ["noted", "take some note", "Tag1", "Tag2", "-o"],
            Command::Note {
                open_after_write: true,
                note: str!("take some note"),
                tags: [str!("Tag1"), str!("Tag2")].to_vec()
            }
        );

        test_command!(
            take_note_with_multiple_tags_open_mixed,
            ["noted", "take some note", "Tag1", "-o", "Tag2", "Tag3"],
            Command::Note {
                open_after_write: true,
                note: str!("take some note"),
                tags: [str!("Tag1"), str!("Tag2"), str!("Tag3")].to_vec()
            }
        );
    }

    mod create_args_test {
        use crate::noted::{Cli, Command};
        use crate::str;

        #[test]
        fn no_args_prints_help_about_required_arguments() {
            let err = Cli::parse(["noted", "create"].iter()).unwrap_err();
            assert!(err
                .message
                .contains("The following required arguments were not provided"));
            assert!(err.message.contains("filename"));
        }

        test_command!(
            create_with_filename,
            ["noted", "create", "test"],
            Command::Create {
                filename: str!("test")
            }
        );

        test_command!(
            alias_c,
            ["noted", "c", "test"],
            Command::Create {
                filename: str!("test")
            }
        );

        test_command!(
            alias_new,
            ["noted", "new", "test"],
            Command::Create {
                filename: str!("test")
            }
        );

        test_command!(
            alias_n,
            ["noted", "n", "test"],
            Command::Create {
                filename: str!("test")
            }
        );
    }

    mod open_args_test {
        use crate::noted::{Cli, Command};
        use crate::str;

        test_command!(
            open_without_filename,
            ["noted", "open"],
            Command::Open { filename: None }
        );

        test_command!(
            open_with_filename,
            ["noted", "open", "file"],
            Command::Open {
                filename: Some(str!("file"))
            }
        );

        test_command!(alias_o, ["noted", "open"], Command::Open { filename: None });

        test_command!(
            alias_edit,
            ["noted", "edit"],
            Command::Open { filename: None }
        );

        test_command!(alias_e, ["noted", "e"], Command::Open { filename: None });

        test_command!(
            alias_view,
            ["noted", "view"],
            Command::Open { filename: None }
        );
    }

    mod search_args_test {
        use crate::noted::{Cli, Command};
        use crate::str;

        #[test]
        fn no_args_prints_help_about_required_arguments() {
            let err = Cli::parse(["noted", "search"].iter()).unwrap_err();
            assert!(err
                .message
                .contains("The following required arguments were not provided"));
            assert!(err.message.contains("pattern"));
        }

        test_command!(
            search_pattern,
            ["noted", "search", "xyz*"],
            Command::Search {
                tag: false,
                pattern: str!("xyz*"),
                file_pattern: None,
                output_to_file: false
            }
        );

        test_command!(
            search_pattern_tag,
            ["noted", "search", "--tag", "xyz*"],
            Command::Search {
                tag: true,
                pattern: str!("xyz*"),
                file_pattern: None,
                output_to_file: false
            }
        );

        test_command!(
            search_pattern_tag_filepattern,
            ["noted", "search", "--tag", "xyz*", "*samplefile*"],
            Command::Search {
                tag: true,
                pattern: str!("xyz*"),
                file_pattern: Some(str!("*samplefile*")),
                output_to_file: false
            }
        );

        test_command!(
            search_pattern_tag_short,
            ["noted", "search", "-t", "xyz*"],
            Command::Search {
                tag: true,
                pattern: str!("xyz*"),
                file_pattern: None,
                output_to_file: false
            }
        );

        test_command!(
            alias_s,
            ["noted", "s", "-t", "xyz*"],
            Command::Search {
                tag: true,
                pattern: str!("xyz*"),
                file_pattern: None,
                output_to_file: false
            }
        );

        test_command!(
            alias_grep,
            ["noted", "grep", "xyz*"],
            Command::Search {
                tag: false,
                pattern: str!("xyz*"),
                file_pattern: None,
                output_to_file: false
            }
        );

        test_command!(
            alias_find,
            ["noted", "find", "xyz*"],
            Command::Search {
                tag: false,
                pattern: str!("xyz*"),
                file_pattern: None,
                output_to_file: false
            }
        );

        test_command!(
            alias_f,
            ["noted", "f", "xyz*"],
            Command::Search {
                tag: false,
                pattern: str!("xyz*"),
                file_pattern: None,
                output_to_file: false
            }
        );
    }

    mod config_test{
        use crate::noted::{Cli, Command};

        test_command!(
            config,
            ["noted", "config"],
            Command::Config
        );
    }

    mod undefined_test{
    // Known Limitaion: Currently it is not possible (at least I didn't find anythin suitable) to disable InferSubcommands in clap.
    // Therefore this bug will persist.
    //     use crate::noted::{Cli, Command};

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
}
