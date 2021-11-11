#[macro_export]
macro_rules! in_temp_dir {
    ($block:block) => {
        let temp_dir = temp_dir::TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir.path()).unwrap();

        $block;
    };
}

#[cfg(test)]
mod file_rolling_tests {
    use std::str::FromStr;

    use crate::noted::FileRolling;

    #[test]
    fn parse_daily() {
        let file_rolling = FileRolling::from_str("Daily").unwrap();
        assert_eq!(FileRolling::Daily, file_rolling);
    }

    #[test]
    fn parse_week() {
        let file_rolling = FileRolling::from_str("Week").unwrap();
        assert_eq!(FileRolling::Week, file_rolling);
    }

    #[test]
    fn parse_month() {
        let file_rolling = FileRolling::from_str("Month").unwrap();
        assert_eq!(FileRolling::Month, file_rolling);
    }

    #[test]
    fn parse_year() {
        let file_rolling = FileRolling::from_str("Year").unwrap();
        assert_eq!(FileRolling::Year, file_rolling);
    }

    #[test]
    fn parse_never() {
        let file_rolling = FileRolling::from_str("Never").unwrap();
        assert_eq!(FileRolling::Never, file_rolling);
    }

    #[test]
    fn parse_case_insensitive() {
        let file_rolling = FileRolling::from_str("week").unwrap();
        assert_eq!(FileRolling::Week, file_rolling);
    }

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
mod command_tests {
    use std::str::FromStr;

    use crate::noted::Command;

    #[test]
    fn parse_default() {
        assert_eq!(Command::Direct, Command::from_str("sample").unwrap());
    }

    #[test]
    fn parse_create() {
        assert_eq!(Command::Create, Command::from_str("create").unwrap());
        assert_eq!(Command::Create, Command::from_str("new").unwrap());
        assert_eq!(Command::Create, Command::from_str("n").unwrap());
    }

    #[test]
    fn parse_config() {
        assert_eq!(Command::Config, Command::from_str("config").unwrap());
    }

    #[test]
    fn parse_open() {
        assert_eq!(Command::Open, Command::from_str("edit").unwrap());
        assert_eq!(Command::Open, Command::from_str("view").unwrap());
        assert_eq!(Command::Open, Command::from_str("open").unwrap());
        assert_eq!(Command::Open, Command::from_str("o").unwrap());
    }

    #[test]
    fn parse_find() {
        assert_eq!(Command::Grep, Command::from_str("grep").unwrap());
        assert_eq!(Command::Grep, Command::from_str("search").unwrap());
        assert_eq!(Command::Grep, Command::from_str("find").unwrap());
        assert_eq!(Command::Grep, Command::from_str("f").unwrap());
    }

    #[test]
    fn parse_version() {
        assert_eq!(Command::Version, Command::from_str("version").unwrap());
        assert_eq!(Command::Version, Command::from_str("v").unwrap());
    }

    #[test]
    fn parse_help() {
        assert_eq!(Command::Help, Command::from_str("help").unwrap());
        assert_eq!(Command::Help, Command::from_str("?").unwrap());
    }
}

#[cfg(test)]
mod note_tests {
    use crate::noted::{Note, LINE_ENDING};
    use crate::{str, vec_of_strings};
    use indoc::indoc;

    #[test]
    fn parse_empty_vec_should_be_default() {
        let args: Vec<String> = Vec::new();
        let note = Note::from(&args);
        assert_eq!(note.content, str!(""));
        assert!(note.tags.is_none());
        assert_eq!(
            format!("%date_format%{0}{0}%note%{0}{0}%tags%", LINE_ENDING),
            note.template.template
        );
        assert_eq!("%F %T", note.template.date_format);
    }

    #[test]
    fn parse_one_argument_should_create_note_without_tags() {
        let args = vec_of_strings!("Sample note");
        let note = Note::from(&args);
        assert_eq!(str!("Sample note"), note.content);
        assert!(note.tags.is_none());
    }

    #[test]
    fn parse_two_arguments_should_create_note_with_tag() {
        let args = vec_of_strings!("Sample note", "tag1");
        let note = Note::from(&args);
        assert_eq!(note.content, str!("Sample note"));
        if let Some(tags) = &note.tags {
            assert_eq!(1, tags.len());
            assert_eq!("tag1", tags[0]);
        }
    }

    #[test]
    fn parse_multiple_arguments_should_create_note_with_multiple_tags() {
        let args = vec_of_strings!("Sample note", "tag1", "tag2", "tag3", "tag4");
        let note = Note::from(&args);
        assert_eq!(str!("Sample note"), note.content);
        assert_eq!(4, note.tags.unwrap().len());
    }

    #[test]
    fn to_string_uses_default_template() {
        let args = vec_of_strings!("Sample note", "tag1", "tag2");
        let note = Note::from(&args);
        let expected = indoc! {"
            Sample note

            #tag1;#tag2

            ---
            "
        };
        assert!(note.to_string().contains(expected));
    }

    #[test]
    fn to_string_uses_template() {
        let args = vec_of_strings!("Sample note", "tag1", "tag2");
        let mut note = Note::from(&args);
        note.template.template = str!(indoc! {
            "%tags%

            %note%"
        });
        let expected = indoc! {"
            #tag1;#tag2

            Sample note

            ---
            "
        };
        assert_eq!(expected, note.to_string());
    }

    #[test]
    fn to_string_uses_date_format() {
        let args = vec_of_strings!("Sample note", "tag1", "tag2");
        let now = chrono::Local::now();
        let year = &now.format("%Y").to_string();
        let mut note = Note::from(&args);
        note.template.date_format = str!("%Y");
        let expected = format!(
            indoc! {"
            {0}

            Sample note

            #tag1;#tag2

            ---
            "
            },
            year
        );
        assert_eq!(expected, note.to_string());
    }

    #[test]
    fn to_string_trims_start_template() {
        let args = vec_of_strings!("Sample note");
        let mut note = Note::from(&args);
        note.template.template = str!(indoc! {
            "%tags%

            %note%"
        });
        let expected = indoc! {"
            Sample note

            ---
            "
        };
        assert_eq!(expected, note.to_string());
    }

    #[test]
    fn to_string_trims_end_template() {
        let args = vec_of_strings!("Sample note");
        let now = chrono::Local::now();
        let year = &now.format("%Y").to_string();
        let mut note = Note::from(&args);
        note.template.date_format = str!("%Y");
        let expected = format!(
            indoc! {"
            {0}

            Sample note

            ---
            "
            },
            year
        );
        assert_eq!(expected, note.to_string());
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
mod markdown_search_arguments_tests {
    use crate::{
        noted::{MarkdownSearchArguments, MarkdownSearchType},
        vec_of_strings,
    };

    #[test]
    fn parse_no_arguments() {
        let search = MarkdownSearchArguments::from([].to_vec());
        assert_eq!("", search.regex);
        assert_eq!(MarkdownSearchType::Default, search.search_type);
        assert!(search.file_regex.is_none());
    }

    #[test]
    fn parse_empty_arguments() {
        let args = vec_of_strings!("");
        let search = MarkdownSearchArguments::from(args);
        assert_eq!("", search.regex);
        assert_eq!(MarkdownSearchType::Default, search.search_type);
        assert!(search.file_regex.is_none());
    }

    #[test]
    fn parse_regex() {
        let args = vec_of_strings!("\\d");
        let search = MarkdownSearchArguments::from(args);
        assert_eq!("\\d", search.regex);
        assert_eq!(MarkdownSearchType::Default, search.search_type);
        assert!(search.file_regex.is_none());
    }

    #[test]
    fn parse_regex_and_file_regex() {
        let args = vec_of_strings!("\\d", "2021-03");
        let search = MarkdownSearchArguments::from(args);
        assert_eq!("\\d", search.regex);
        assert_eq!(MarkdownSearchType::Default, search.search_type);
        assert!(search.file_regex.is_some());
        assert_eq!("2021-03", search.file_regex.unwrap());
    }

    #[test]
    fn parse_search_type_tags_and_regex() {
        let args = vec_of_strings!("t", "\\d");
        let search = MarkdownSearchArguments::from(args);
        assert_eq!("\\d", search.regex);
        assert_eq!(MarkdownSearchType::Tags, search.search_type);
        assert!(search.file_regex.is_none());
    }

    #[test]
    fn parse_search_type_tags_without_regex() {
        let args = vec_of_strings!("t");
        let search = MarkdownSearchArguments::from(args);
        assert_eq!("", search.regex);
        assert_eq!(MarkdownSearchType::Tags, search.search_type);
        assert!(search.file_regex.is_none());
    }
}

#[cfg(test)]
mod markdown_search_type_tests {
    use std::str::FromStr;

    use crate::noted::MarkdownSearchType;

    #[test]
    fn parse_tag() {
        assert_eq!(
            MarkdownSearchType::Tags,
            MarkdownSearchType::from_str("tag").unwrap()
        );
        assert_eq!(
            MarkdownSearchType::Tags,
            MarkdownSearchType::from_str("tags").unwrap()
        );
        assert_eq!(
            MarkdownSearchType::Tags,
            MarkdownSearchType::from_str("t").unwrap()
        );
    }

    #[test]
    fn parse_default() {
        assert_eq!(
            MarkdownSearchType::Default,
            MarkdownSearchType::from_str("other").unwrap()
        );
        assert_eq!(
            MarkdownSearchType::Default,
            MarkdownSearchType::from_str("l").unwrap()
        );
    }
}

#[cfg(test)]
mod markdown_search_result_tests {
    use crate::noted::MarkdownSearchResult;
    use crate::str;

    #[test]
    fn format_short_filename_no_dots() {
        let data: Vec<(String, u64, String)> =
            vec![(str!("/temp/long/filename.md"), 1, str!("test"))];
        // let expected = MarkdownSearchResult::fmt("/temp/long/filename.md", "1", "test");
        let expected = "/temp/long/filename.md         | 1    | test                                         ";

        //ACT
        let table = MarkdownSearchResult::to_table(data);
        //ASSERT
        assert!(!table[1].starts_with("..."));
        assert_eq!(expected, table[1]);
    }

    #[test]
    fn format_long_filename_with_dots() {
        let data: Vec<(String, u64, String)> =
            vec![(str!("/temp/long/and/longer/or/evenlonger/filename.md"), 1, str!("test"))];
        // let expected = MarkdownSearchResult::fmt("...r/or/evenlonger/filename.md", "1", "test");
        let expected = "...r/or/evenlonger/filename.md | 1    | test                                         ";

        //ACT
        let table = MarkdownSearchResult::to_table(data);
        //ASSERT
        assert!(table[1].starts_with("..."));
        assert_eq!(expected, table[1]);
    }

    #[test]
    fn format_line_number_four_digits() {
        let data: Vec<(String, u64, String)> =
            vec![(str!("/temp/long/and/longer/or/evenlonger/filename.md"), 9999, str!("test"))];
        // let expected = MarkdownSearchResult::fmt("...r/or/evenlonger/filename.md", "9999", "test");
        let expected = "...r/or/evenlonger/filename.md | 9999 | test                                         ";
        //ACT
        let table = MarkdownSearchResult::to_table(data);
        //ASSERT
        assert!(table[1].starts_with("..."));
        assert_eq!(expected, table[1]);
    }

    #[test]
    fn format_line_number_five_digits() {
        let data: Vec<(String, u64, String)> =
            vec![(str!("/temp/long/and/longer/or/evenlonger/filename.md"), 99999, str!("test"))];
        // let expected = MarkdownSearchResult::fmt("...r/or/evenlonger/filename.md", "99999", "test");
        let expected = "...r/or/evenlonger/filename.md | 99999 | test                                         ";
        //ACT
        let table = MarkdownSearchResult::to_table(data);
        //ASSERT
        assert!(table[1].starts_with("..."));
        assert_eq!(expected, table[1]);
    }

    #[test]
    fn format_long_line() {
        let data: Vec<(String, u64, String)> =
            vec![(str!("/temp/long/and/longer/or/evenlonger/filename.md"), 9999, str!("Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum."))];
        // let expected = MarkdownSearchResult::fmt("...r/or/evenlonger/filename.md", "99999", "test");
        let expected = "...r/or/evenlonger/filename.md | 9999 | Lorem Ipsum is simply dummy text of the pr...";
        //ACT
        let table = MarkdownSearchResult::to_table(data);
        //ASSERT
        assert!(table[1].starts_with("..."));
        assert_eq!(expected, table[1]);
    }

}

#[cfg(test)]
mod markdown_tests {

    mod write_tests {
        use crate::noted::{Configuration, Markdown, Note, PostCommand, NOTES_FILE_NAME};
        use crate::{str, vec_of_strings};
        use indoc::indoc;

        #[test]
        #[serial_test::serial]
        fn custom_filename() {
            in_temp_dir!({
                let cur_dir = std::env::current_dir().unwrap();
                let configuration = Configuration {
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                let note = Note::from(&vec_of_strings!("sample note"));
                let file = Markdown::custom_target(&[str!("sample")], &configuration);

                match Markdown::write(&note, &file) {
                    Ok(_) => {
                        let expected = cur_dir.join("sample.md");
                        assert!(expected.exists());
                    }
                    Err(_) => panic!("write note month failed"),
                };
                Markdown::write(&note, &file).unwrap();
                let options = glob::MatchOptions {
                    case_sensitive: false,
                    ..Default::default()
                };
                assert!(glob::glob_with("*", options).unwrap().count() == 1);
            });
        }

        #[test]
        #[serial_test::serial]
        fn daily_rolling() {
            in_temp_dir!({
                let cur_dir = std::env::current_dir().unwrap();
                let configuration = Configuration {
                    file_rolling: crate::noted::FileRolling::Daily,
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                let note = Note::from(&vec_of_strings!("sample note"));
                let file = Markdown::target(&configuration);

                match Markdown::write(&note, &file) {
                    Ok(_) => {
                        let now = chrono::Local::now();
                        let file_name = format!("{}.md", &now.format("%Y-%m-%d"));
                        let expected = cur_dir.join(file_name);
                        assert!(expected.exists());
                    }
                    Err(_) => panic!("write note month failed"),
                };
                Markdown::write(&note, &file).unwrap();
                let options = glob::MatchOptions {
                    case_sensitive: false,
                    ..Default::default()
                };
                assert!(glob::glob_with("*", options).unwrap().count() == 1);
            });
        }

        #[test]
        #[serial_test::serial]
        fn month_rolling() {
            in_temp_dir!({
                let cur_dir = std::env::current_dir().unwrap();
                let configuration = Configuration {
                    file_rolling: crate::noted::FileRolling::Month,
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                let note = Note::from(&vec_of_strings!("sample note"));
                let file = Markdown::target(&configuration);

                match Markdown::write(&note, &file) {
                    Ok(_) => {
                        let now = chrono::Local::now();
                        let file_name = format!("{}.md", &now.format("%Y-%m"));
                        let expected = cur_dir.join(file_name);
                        assert!(expected.exists());
                    }
                    Err(_) => panic!("write note month failed"),
                };
                Markdown::write(&note, &file).unwrap();
                let options = glob::MatchOptions {
                    case_sensitive: false,
                    ..Default::default()
                };
                assert!(glob::glob_with("*", options).unwrap().count() == 1);
            });
        }

        #[test]
        #[serial_test::serial]
        fn week_rolling() {
            in_temp_dir!({
                let cur_dir = std::env::current_dir().unwrap();
                let configuration = Configuration {
                    file_rolling: crate::noted::FileRolling::Week,
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                let note = Note::from(&vec_of_strings!("sample note"));
                let file = Markdown::target(&configuration);

                match Markdown::write(&note, &file) {
                    Ok(_) => {
                        let now = chrono::Local::now();
                        let file_name = format!("{}.md", &now.format("%Y-%W"));
                        let expected = cur_dir.join(file_name);
                        assert!(expected.exists());
                    }
                    Err(_) => panic!("write note month failed"),
                };
                Markdown::write(&note, &file).unwrap();
                let options = glob::MatchOptions {
                    case_sensitive: false,
                    ..Default::default()
                };
                assert!(glob::glob_with("*", options).unwrap().count() == 1);
            });
        }

        #[test]
        #[serial_test::serial]
        fn year_rolling() {
            in_temp_dir!({
                let cur_dir = std::env::current_dir().unwrap();
                let configuration = Configuration {
                    file_rolling: crate::noted::FileRolling::Year,
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                let note = Note::from(&vec_of_strings!("sample note"));
                let file = Markdown::target(&configuration);

                match Markdown::write(&note, &file) {
                    Ok(_) => {
                        let now = chrono::Local::now();
                        let file_name = format!("{}.md", &now.format("%Y"));
                        let expected = cur_dir.join(file_name);
                        assert!(expected.exists());
                    }
                    Err(_) => panic!("write note month failed"),
                };
                Markdown::write(&note, &file).unwrap();
                let options = glob::MatchOptions {
                    case_sensitive: false,
                    ..Default::default()
                };
                assert!(glob::glob_with("*", options).unwrap().count() == 1);
            });
        }

        #[test]
        #[serial_test::serial]
        fn note_strucutre() {
            in_temp_dir!({
                let cur_dir = std::env::current_dir().unwrap();
                let note_template = str!(indoc! {
                    "%note%

                    %tags%"
                });
                let configuration = Configuration {
                    file_rolling: crate::noted::FileRolling::Year,
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                let file = Markdown::target(&configuration);

                let mut note = Note::from(&vec_of_strings!("sample note"));
                note.template.template = note_template.clone();
                Markdown::write(&note, &file).unwrap();

                let mut note2 = Note::from(&vec_of_strings!("sample note 2", "tag"));
                note2.template.template = note_template.clone();
                Markdown::write(&note2, &file).unwrap();

                let mut note3 = Note::from(&vec_of_strings!("sample note 3"));
                note3.template.template = note_template;
                Markdown::write(&note3, &file).unwrap();

                let expected_note_content = vec_of_strings!(
                    "sample note",
                    "",
                    "---",
                    "sample note 2",
                    "",
                    "#tag",
                    "",
                    "---",
                    "sample note 3",
                    "",
                    "---",
                    ""
                );

                let read_note_file = std::fs::File::open(file).unwrap();
                let raw_note: Vec<String> =
                    std::io::BufRead::lines(std::io::BufReader::new(read_note_file))
                        .map(|l| l.unwrap())
                        .collect();

                for i in 0..raw_note.len() - 1 {
                    assert_eq!(expected_note_content[i], raw_note[i]);
                }
            });
        }

        #[test]
        #[serial_test::serial]
        fn mixed_rolling() {
            in_temp_dir!({
                let cur_dir = std::env::current_dir().unwrap();
                // Write Daily note.
                let configuration = Configuration {
                    file_rolling: crate::noted::FileRolling::Daily,
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                let note = Note::from(&vec_of_strings!("sample note"));
                let file1 = Markdown::target(&configuration);

                Markdown::write(&note, &file1).unwrap();

                // Change to week and write note
                let configuration2 = Configuration {
                    file_rolling: crate::noted::FileRolling::Week,
                    ..configuration
                };
                let file2 = Markdown::target(&configuration2);

                Markdown::write(&note, &file2).unwrap();

                // Change to never and write note
                let configuration3 = Configuration {
                    file_rolling: crate::noted::FileRolling::Never,
                    ..configuration2
                };
                let file3 = Markdown::target(&configuration3);

                Markdown::write(&note, &file3).unwrap();

                let options = glob::MatchOptions {
                    case_sensitive: false,
                    ..Default::default()
                };
                assert!(glob::glob_with("*", options).unwrap().count() == 3);

                let expected = cur_dir.join(NOTES_FILE_NAME);
                assert!(expected.exists());
            });
        }

        #[test]
        #[serial_test::serial]
        fn write_and_open() {
            in_temp_dir!({
                let cur_dir = std::env::current_dir().unwrap();
                let configuration = Configuration {
                    file_rolling: crate::noted::FileRolling::Daily,
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                let note = Note::from(&vec_of_strings!("sample note", "tag", "-o"));
                if let Some(tags) = &note.tags {
                    assert_eq!(1, tags.len());
                }
                let file = Markdown::target(&configuration);

                match Markdown::write(&note, &file) {
                    Ok(res) => {
                        let now = chrono::Local::now();
                        let file_name = format!("{}.md", &now.format("%Y-%m-%d"));
                        let expected = cur_dir.join(file_name);
                        assert!(expected.exists());
                        assert_eq!(PostCommand::Open, res);
                    }
                    Err(_) => panic!("write note month failed"),
                };
                Markdown::write(&note, &file).unwrap();
                let options = glob::MatchOptions {
                    case_sensitive: false,
                    ..Default::default()
                };
                assert!(glob::glob_with("*", options).unwrap().count() == 1);
            });
        }

        #[test]
        #[serial_test::serial]
        fn write_and_do_nothing() {
            in_temp_dir!({
                let cur_dir = std::env::current_dir().unwrap();
                let configuration = Configuration {
                    file_rolling: crate::noted::FileRolling::Daily,
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                let note = Note::from(&vec_of_strings!("sample note", "tag"));
                let file = Markdown::target(&configuration);

                match Markdown::write(&note, &file) {
                    Ok(res) => {
                        let now = chrono::Local::now();
                        let file_name = format!("{}.md", &now.format("%Y-%m-%d"));
                        let expected = cur_dir.join(file_name);
                        assert!(expected.exists());
                        assert_eq!(PostCommand::None, res);
                    }
                    Err(_) => panic!("write note month failed"),
                };
                Markdown::write(&note, &file).unwrap();
                let options = glob::MatchOptions {
                    case_sensitive: false,
                    ..Default::default()
                };
                assert!(glob::glob_with("*", options).unwrap().count() == 1);
            });
        }
    }

    mod search_tests {
        use crate::{
            noted::{
                Configuration, FileRolling, Markdown, MarkdownSearchArguments,
                MarkdownSearchResult, Note,
            },
            str, vec_of_strings,
        };

        #[test]
        fn empty_search_arguments_invalidinput_error() {
            let search = MarkdownSearchArguments::from(vec_of_strings!("t"));
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
            let search = MarkdownSearchArguments::from(vec_of_strings!("t", "bug", "_\\"));
            let configuration = Configuration::default();
            match Markdown::search(search, &configuration) {
                Ok(_) => panic!("should not be ok"),
                Err(err) => {
                    assert_eq!(std::io::ErrorKind::NotFound, err.kind());
                }
            }
        }

        // test mit fallback auf configuration

        // test mit einzelner datei gefunden
        #[test]
        #[serial_test::serial]
        fn find_single_occurence_in_one_file() {
            in_temp_dir!({
                // ARRANGE
                let cur_dir = std::env::current_dir().unwrap();
                let configuration = Configuration {
                    note_directory: str!(cur_dir, PathBuf),
                    file_rolling: FileRolling::Never,
                    ..Default::default()
                };
                let note = Note::from(&vec_of_strings!("sample note"));
                let file = Markdown::target(&configuration);
                Markdown::write(&note, &file).unwrap();
                //ACT
                let search_arguments = MarkdownSearchArguments {
                    regex: str!(".*note"),
                    ..Default::default()
                };
                let result = Markdown::search(search_arguments, &configuration);
                assert!(result.is_ok());
                MarkdownSearchResult::to_table(result.unwrap());
            });
        }

        // test mit mehreren dateien gefunden
    }

    mod taget_by_pattern_tests {
        use crate::noted::{Configuration, Markdown};
        use crate::{str, vec_of_strings};

        #[test]
        #[serial_test::serial]
        fn empty_argument_returns_fallback() {
            in_temp_dir!({
                // ARAMGE
                let cur_dir = std::env::current_dir().unwrap();
                let now = chrono::Local::now();
                let file_name = format!("{}.md", &now.format("%Y"));
                std::fs::File::create(&file_name).unwrap();
                let configuration = Configuration {
                    file_rolling: crate::noted::FileRolling::Year,
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                let test_args = vec_of_strings!("callstack");
                // ACT
                if let Some(target) =
                    Markdown::taget_by_pattern(test_args[1..].to_vec(), &configuration)
                {
                    // ASSERT
                    assert_eq!(
                        format!("{}/{}", str!(cur_dir, PathBuf), file_name),
                        str!(target, PathBuf)
                    );
                } else {
                    panic!("no file found!");
                }
            });
        }

        #[test]
        #[serial_test::serial]
        fn no_matching_file_is_found_returns_none() {
            in_temp_dir!({
                // ARAMGE
                let cur_dir = std::env::current_dir().unwrap();
                let now = chrono::Local::now();
                let file_name = format!("{}.md", &now.format("%Y"));
                std::fs::File::create(&file_name).unwrap();
                let configuration = Configuration {
                    file_rolling: crate::noted::FileRolling::Year,
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                // ACT
                let target = Markdown::taget_by_pattern(vec_of_strings!("04"), &configuration);
                assert!(target.is_none());
            });
        }

        #[test]
        #[serial_test::serial]
        fn find_by_filename() {
            in_temp_dir!({
                // ARAMGE
                let cur_dir = std::env::current_dir().unwrap();
                let now = chrono::Local::now();
                let cur_year = &now.format("%Y");
                let file_name = format!("{}.md", &now.format("%Y"));
                std::fs::File::create(&file_name).unwrap();
                let configuration = Configuration {
                    file_rolling: crate::noted::FileRolling::Year,
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                // ACT
                if let Some(target) =
                    Markdown::taget_by_pattern(vec_of_strings!(cur_year), &configuration)
                {
                    // ASSERT
                    assert_eq!(
                        format!("{}/{}", str!(cur_dir, PathBuf), file_name),
                        str!(target, PathBuf)
                    );
                } else {
                    panic!("no file found!");
                }
            });
        }

        #[test]
        #[serial_test::serial]
        fn find_by_filename_wildcard_single_file() {
            in_temp_dir!({
                // ARAMGE
                let cur_dir = std::env::current_dir().unwrap();
                let now = chrono::Local::now();
                let file_name = format!("{}.md", &now.format("%Y"));
                std::fs::File::create(&file_name).unwrap();
                let configuration = Configuration {
                    file_rolling: crate::noted::FileRolling::Year,
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                let pattern = format!("*{}", &now.format("%y"));
                // ACT
                if let Some(target) =
                    Markdown::taget_by_pattern(vec_of_strings!(pattern), &configuration)
                {
                    // ASSERT
                    assert_eq!(
                        format!("{}/{}", str!(cur_dir, PathBuf), file_name),
                        str!(target, PathBuf)
                    );
                } else {
                    panic!("no file found!");
                }
            });
        }

        #[test]
        #[serial_test::serial]
        fn find_by_filename_wildcard_no_char_before_ext() {
            in_temp_dir!({
                // ARAMGE
                let cur_dir = std::env::current_dir().unwrap();
                std::fs::File::create("2021-02.md").unwrap();
                std::fs::File::create("2021-04.md").unwrap();
                let configuration = Configuration {
                    file_rolling: crate::noted::FileRolling::Daily,
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                // ACT
                if let Some(target) =
                    Markdown::taget_by_pattern(vec_of_strings!("2021-04*"), &configuration)
                {
                    // ASSERT
                    assert_eq!(
                        format!("{}/{}", str!(cur_dir, PathBuf), "2021-04.md"),
                        str!(target, PathBuf)
                    );
                } else {
                    panic!("no file found!");
                }
            });
        }

        #[test]
        #[serial_test::serial]
        fn find_by_filename_wildcard_multiple_files_rolling_mix() {
            in_temp_dir!({
                // ARAMGE
                let cur_dir = std::env::current_dir().unwrap();
                std::fs::File::create("2021-02.md").unwrap();
                std::fs::File::create("2021-04-02.md").unwrap();
                std::fs::File::create("2021-04-03.md").unwrap();
                std::fs::File::create("2021-04.md").unwrap();
                let configuration = Configuration {
                    file_rolling: crate::noted::FileRolling::Daily,
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                // ACT
                if let Some(target) =
                    Markdown::taget_by_pattern(vec_of_strings!("2021-04*"), &configuration)
                {
                    // ASSERT
                    assert_eq!(
                        format!("{}/{}", str!(cur_dir, PathBuf), "2021-04-02.md"),
                        str!(target, PathBuf)
                    );
                } else {
                    panic!("no file found!");
                }
            });
        }

        #[test]
        #[serial_test::serial]
        fn find_by_filename_wildcard_multiple_files_sorting() {
            in_temp_dir!({
                // ARAMGE
                let cur_dir = std::env::current_dir().unwrap();
                std::fs::File::create("2021-02.md").unwrap();
                std::fs::File::create("2021-04-02.md").unwrap();
                std::fs::File::create("2021-04-03.md").unwrap();
                std::fs::File::create("2021-01.md").unwrap();
                let configuration = Configuration {
                    file_rolling: crate::noted::FileRolling::Daily,
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                // ACT
                if let Some(target) =
                    Markdown::taget_by_pattern(vec_of_strings!("2021*"), &configuration)
                {
                    // ASSERT
                    assert_eq!(
                        format!("{}/{}", str!(cur_dir, PathBuf), "2021-01.md"),
                        str!(target, PathBuf)
                    );
                } else {
                    panic!("no file found!");
                }
            });
        }
    }

    mod target_tests {
        use crate::noted::{Configuration, Markdown};
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
            let target = Markdown::target(&configuration);
            // ASSERT
            assert!(str!(target, PathBuf).starts_with("/home"));
            assert!(str!(target, PathBuf).ends_with(&file_name));
        }

        #[test]
        #[serial_test::serial]
        fn repository_specific_no_repository() {
            in_temp_dir!({
                let cur_dir = std::env::current_dir().unwrap();
                let configuration = Configuration {
                    use_repository_specific: true,
                    ..Default::default()
                };
                let target = Markdown::target(&configuration);
                assert_ne!(
                    format!("{}/notes.md", cur_dir.to_str().unwrap()),
                    str!(target, PathBuf)
                );
                assert!(str!(target, PathBuf).starts_with("/home"));
            });
        }

        #[test]
        #[serial_test::serial]
        fn repository_specific_at_root() {
            in_temp_dir!({
                let cur_dir = std::env::current_dir().unwrap();
                git2::Repository::init(&cur_dir).unwrap();

                // dbg!(&cur_dir);
                let configuration = Configuration {
                    use_repository_specific: true,
                    ..Default::default()
                };
                let target = Markdown::target(&configuration);
                assert_eq!(
                    format!("{}/notes.md", cur_dir.to_str().unwrap()),
                    str!(target, PathBuf)
                );
            });
        }

        #[test]
        #[serial_test::serial]
        fn repository_specific_at_subfolder() {
            in_temp_dir!({
                let current_dir = std::env::current_dir().unwrap();
                git2::Repository::init(&current_dir).unwrap();
                let sub_dir = current_dir.join("src").join("module");
                std::fs::create_dir_all(&sub_dir).unwrap();
                std::env::set_current_dir(&sub_dir).unwrap();
                let cur_sub_dir = std::env::current_dir().unwrap();

                // dbg!(&cur_sub_dir);
                assert_eq!(sub_dir, cur_sub_dir);

                let configuration = Configuration {
                    use_repository_specific: true,
                    ..Default::default()
                };
                let target = Markdown::target(&configuration);
                assert_eq!(
                    format!("{}/notes.md", current_dir.to_str().unwrap()),
                    str!(target, PathBuf)
                );
            });
        }
    }

    mod custom_target_tests {
        use crate::{
            noted::{Configuration, FileRolling, Markdown, NOTES_FILE_NAME},
            str,
        };

        #[test]
        fn file_with_md_extension() {
            in_temp_dir!({
                // ARAMGE
                let cur_dir = std::env::current_dir().unwrap();
                let configuration = Configuration {
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                let target = Markdown::custom_target(&[str!("test.md")], &configuration);
                assert_eq!(cur_dir.join("test.md"), target);
            });
        }

        #[test]
        fn file_without_md_extension() {
            in_temp_dir!({
                // ARAMGE
                let cur_dir = std::env::current_dir().unwrap();
                let configuration = Configuration {
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                let target = Markdown::custom_target(&[str!("test")], &configuration);
                assert_eq!(cur_dir.join("test.md"), target);
            });
        }

        #[test]
        fn file_with_other_extension() {
            in_temp_dir!({
                // ARAMGE
                let cur_dir = std::env::current_dir().unwrap();
                let configuration = Configuration {
                    note_directory: str!(cur_dir, PathBuf),
                    ..Default::default()
                };
                let target = Markdown::custom_target(&[str!("test.ini")], &configuration);
                assert_eq!(cur_dir.join("test.ini.md"), target);
            });
        }

        #[test]
        fn file_without_name_returns_from_config() {
            in_temp_dir!({
                // ARAMGE
                let cur_dir = std::env::current_dir().unwrap();
                let configuration = Configuration {
                    note_directory: str!(cur_dir, PathBuf),
                    file_rolling: FileRolling::Never,
                    ..Default::default()
                };
                let target = Markdown::custom_target(&[], &configuration);
                assert_eq!(cur_dir.join(NOTES_FILE_NAME), target);
            });
        }
    }
}
