#[macro_export]
macro_rules! assert_file_exists {
    ($($x:expr),*) => {
        $({
            for _i in [1..5]{
                if $x.exists(){
                    break
                } else {
                    wait!("1");
                }
            }
        })*
    };
}

#[macro_export]
macro_rules! wait {
    ($($x:expr),*) => {
        $({
        let mut child = std::process::Command::new("sleep").arg($x).spawn().unwrap();
        let _result = child.wait().unwrap();
        })*
    };
}

#[macro_export]
macro_rules! safe_file_create{
    ($($x:expr),*) => {
        $({
            std::fs::File::create($x).unwrap();
            wait_file_created::robust_wait_read($x).unwrap();
        })*
    };
}

#[cfg(test)]
mod markdown_tests {

    mod search_tests {
        use crate::{
            noted::{
                Configuration, FileRolling, Markdown, Note, NoteFile, SearchArguments,
                NOTES_FILE_NAME,
            },
            str,
        };

        struct SearchContext {
            temp_dir: tempfile::TempDir,
        }

        impl test_context::TestContext for SearchContext {
            fn setup() -> SearchContext {
                let temp_dir = tempfile::tempdir().unwrap();
                std::env::set_current_dir(&temp_dir.path()).unwrap();
                SearchContext { temp_dir }
            }

            fn teardown(self) {
                self.temp_dir.close().unwrap();
            }
        }

        #[test_context::test_context(SearchContext)]
        #[test]
        #[serial_test::serial]
        fn find_single_occurence_in_one_file(ctx: &mut SearchContext) {
            // ARRANGE
            let cur_dir = std::env::current_dir().unwrap();
            let configuration = Configuration {
                note_directory: str!(cur_dir, PathBuf),
                file_rolling: FileRolling::Never,
                ..Default::default()
            };
            let note = Note::from("sample note").format(&configuration.note_template);
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
            //ACT
            let search_arguments = SearchArguments {
                regex: str!(".*note"),
                ..Default::default()
            };
            match Markdown::search(search_arguments, &configuration) {
                Ok(res) => {
                    // ASSERT
                    assert_eq!(
                        (
                            str!(cur_dir.join(NOTES_FILE_NAME), PathBuf),
                            3,
                            "sample note".to_string()
                        ),
                        res[0]
                    );
                }
                Err(err) => panic!("find in single file error: {:?}", err),
            };
        }

        #[test_context::test_context(SearchContext)]
        #[test]
        #[serial_test::serial]
        fn find_multiple_occurences_in_one_file(ctx: &mut SearchContext) {
            // ARRANGE
            let cur_dir = std::env::current_dir().unwrap();
            let configuration = Configuration {
                note_directory: str!(cur_dir, PathBuf),
                file_rolling: FileRolling::Never,
                ..Default::default()
            };
            let note = Note::from("sample note").format(&configuration.note_template);
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
            //ACT
            let search_arguments = SearchArguments {
                regex: str!(".*note"),
                ..Default::default()
            };
            match Markdown::search(search_arguments, &configuration) {
                Ok(res) => {
                    // ASSERT
                    assert_eq!(2, res.len());
                    assert_eq!(
                        (
                            str!(cur_dir.join(NOTES_FILE_NAME), PathBuf),
                            3,
                            "sample note".to_string()
                        ),
                        res[0]
                    );
                    assert_eq!(
                        (
                            str!(cur_dir.join(NOTES_FILE_NAME), PathBuf),
                            8,
                            "sample note".to_string()
                        ),
                        res[1]
                    );
                }
                Err(err) => panic!("finds in single file error: {:?}", err),
            };
        }

        #[test_context::test_context(SearchContext)]
        #[test]
        #[serial_test::serial]
        fn find_sinle_occurence_in_multiple_files(ctx: &mut SearchContext) {
            // ARRANGE
            let cur_dir = std::env::current_dir().unwrap();

            let mut configuration = Configuration {
                note_directory: str!(cur_dir, PathBuf),
                file_rolling: FileRolling::Never,
                ..Default::default()
            };
            let note = Note::from("sample note").format(&configuration.note_template);
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());

            configuration.file_rolling = FileRolling::Year;
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
            //ACT
            let search_arguments = SearchArguments {
                regex: str!(".*note"),
                ..Default::default()
            };
            match Markdown::search(search_arguments, &configuration) {
                Ok(res) => {
                    // ASSERT
                    let now = chrono::Local::now();
                    let file_name = format!("{}.md", &now.format("%Y"));
                    assert_eq!(2, res.len());
                    assert_eq!(
                        (
                            str!(cur_dir.join(file_name), PathBuf),
                            3,
                            "sample note".to_string()
                        ),
                        res[0]
                    );
                    assert_eq!(
                        (
                            str!(cur_dir.join(NOTES_FILE_NAME), PathBuf),
                            3,
                            "sample note".to_string()
                        ),
                        res[1]
                    );
                }
                Err(err) => panic!("find in multiple files error: {:?}", err),
            };
        }

        #[test_context::test_context(SearchContext)]
        #[test]
        #[serial_test::serial]
        fn find_multiple_occurence_in_multiple_files(ctx: &mut SearchContext) {
            // ARRANGE
            let cur_dir = std::env::current_dir().unwrap();

            let mut configuration = Configuration {
                note_directory: str!(cur_dir, PathBuf),
                file_rolling: FileRolling::Never,
                ..Default::default()
            };
            let note = Note::from("sample note").format(&configuration.note_template);
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());

            configuration.file_rolling = FileRolling::Year;
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
            //ACT
            let search_arguments = SearchArguments {
                regex: str!(".*note"),
                ..Default::default()
            };
            match Markdown::search(search_arguments, &configuration) {
                Ok(res) => {
                    // ASSERT
                    let now = chrono::Local::now();
                    let file_name = format!("{}.md", &now.format("%Y"));
                    assert_eq!(4, res.len());
                    assert_eq!(
                        (
                            str!(cur_dir.join(&file_name), PathBuf),
                            3,
                            "sample note".to_string()
                        ),
                        res[0]
                    );
                    assert_eq!(
                        (
                            str!(cur_dir.join(&file_name), PathBuf),
                            8,
                            "sample note".to_string()
                        ),
                        res[1]
                    );
                    assert_eq!(
                        (
                            str!(cur_dir.join(NOTES_FILE_NAME), PathBuf),
                            3,
                            "sample note".to_string()
                        ),
                        res[2]
                    );
                    assert_eq!(
                        (
                            str!(cur_dir.join(NOTES_FILE_NAME), PathBuf),
                            8,
                            "sample note".to_string()
                        ),
                        res[3]
                    );
                }
                Err(err) => panic!("find in multiple files error: {:?}", err),
            };
        }

        #[test_context::test_context(SearchContext)]
        #[test]
        #[serial_test::serial]
        fn find_tags(ctx: &mut SearchContext) {
            // ARRANGE
            let cur_dir = std::env::current_dir().unwrap();
            let mut configuration = Configuration {
                note_directory: str!(cur_dir, PathBuf),
                file_rolling: FileRolling::Never,
                ..Default::default()
            };
            let note = Note::from(["sample note", "test", "test1"].to_vec())
                .format(&configuration.note_template);
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
            configuration.file_rolling = FileRolling::Year;
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
            //ACT
            let search_arguments = SearchArguments {
                regex: str!("test"),
                tags_only: true,
                ..Default::default()
            };
            match Markdown::search(search_arguments, &configuration) {
                Ok(res) => {
                    // ASSERT
                    let now = chrono::Local::now();
                    let file_name = format!("{}.md", &now.format("%Y"));
                    assert_eq!(2, res.len());
                    assert_eq!(
                        (
                            str!(cur_dir.join(&file_name), PathBuf),
                            5,
                            "#test".to_string()
                        ),
                        res[0]
                    );
                    assert_eq!(
                        (
                            str!(cur_dir.join(NOTES_FILE_NAME), PathBuf),
                            5,
                            "#test".to_string()
                        ),
                        res[1]
                    );
                }
                Err(err) => panic!("find tags error: {:?}", err),
            };
        }

        #[test_context::test_context(SearchContext)]
        #[test]
        #[serial_test::serial]
        fn find_multiple_occurence_in_filtered_files(ctx: &mut SearchContext) {
            // ARRANGE
            let cur_dir = std::env::current_dir().unwrap();

            let mut configuration = Configuration {
                note_directory: str!(cur_dir, PathBuf),
                file_rolling: FileRolling::Never,
                ..Default::default()
            };
            let note = Note::from("sample note").format(&configuration.note_template);
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());

            configuration.file_rolling = FileRolling::Year;
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
            //ACT
            let search_arguments = SearchArguments {
                regex: str!(".*note"),
                file_regex: Some("note*".to_string()),
                ..Default::default()
            };
            match Markdown::search(search_arguments, &configuration) {
                Ok(res) => {
                    // ASSERT
                    assert_eq!(2, res.len());
                    assert_eq!(
                        (
                            str!(cur_dir.join(NOTES_FILE_NAME), PathBuf),
                            3,
                            "sample note".to_string()
                        ),
                        res[0]
                    );
                    assert_eq!(
                        (
                            str!(cur_dir.join(NOTES_FILE_NAME), PathBuf),
                            8,
                            "sample note".to_string()
                        ),
                        res[1]
                    );
                }
                Err(err) => panic!("find in multiple files error: {:?}", err),
            };
        }

        #[test_context::test_context(SearchContext)]
        #[test]
        #[serial_test::serial]
        fn nothing_found(ctx: &mut SearchContext) {
            // ARRANGE
            let cur_dir = std::env::current_dir().unwrap();
            let configuration = Configuration {
                note_directory: str!(cur_dir, PathBuf),
                file_rolling: FileRolling::Never,
                ..Default::default()
            };
            let note = Note::from("sample note").format(&configuration.note_template);
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
            //ACT
            let search_arguments = SearchArguments {
                regex: str!(".*abc"),
                ..Default::default()
            };
            match Markdown::search(search_arguments, &configuration) {
                Ok(res) => {
                    // ASSERT
                    assert!(res.is_empty());
                }
                Err(err) => panic!("find in single file error: {:?}", err),
            };
        }

        #[test_context::test_context(SearchContext)]
        #[test]
        #[serial_test::serial]
        fn focus_on_occurence_in_file(ctx: &mut SearchContext) {
            // ARRANGE
            let cur_dir = std::env::current_dir().unwrap();
            let configuration = Configuration {
                note_directory: str!(cur_dir, PathBuf),
                file_rolling: FileRolling::Never,
                ..Default::default()
            };
            let note = Note::from("sample note which is taken only for this test.")
                .format(&configuration.note_template);
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
            //ACT
            let search_arguments = SearchArguments {
                regex: str!("note"),
                ..Default::default()
            };
            match Markdown::search(search_arguments, &configuration) {
                Ok(res) => {
                    // ASSERT
                    assert_eq!(
                        (
                            str!(cur_dir.join(NOTES_FILE_NAME), PathBuf),
                            3,
                            "sample note which is taken only".to_string()
                        ),
                        res[0]
                    );
                }
                Err(err) => panic!("find in single file error: {:?}", err),
            };
        }

        #[test_context::test_context(SearchContext)]
        #[test]
        #[serial_test::serial]
        fn focus_on_occurence_long_in_file(ctx: &mut SearchContext) {
            // ARRANGE
            let cur_dir = std::env::current_dir().unwrap();
            let configuration = Configuration {
                note_directory: str!(cur_dir, PathBuf),
                file_rolling: FileRolling::Never,
                ..Default::default()
            };
            let note = Note::from("Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.").format(&configuration.note_template);
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
            //ACT
            let search_arguments = SearchArguments {
                regex: str!("centuries"),
                ..Default::default()
            };
            match Markdown::search(search_arguments, &configuration) {
                Ok(res) => {
                    // ASSERT
                    assert_eq!(
                        (
                            str!(cur_dir.join(NOTES_FILE_NAME), PathBuf),
                            3,
                            "ved not only five centuries, but also the lea".to_string()
                        ),
                        res[0]
                    );
                }
                Err(err) => panic!("find in single file error: {:?}", err),
            };
        }
    }

    mod write_tests {
        use crate::noted::{
            Configuration, Markdown, Note, NoteFile, NoteTemplate, NOTES_FILE_NAME,
        };
        use crate::{str, vec_of_strings};
        use indoc::indoc;

        struct WriteContext {
            temp_dir: tempfile::TempDir,
        }

        impl test_context::TestContext for WriteContext {
            fn setup() -> WriteContext {
                let temp_dir = tempfile::tempdir().unwrap();
                std::env::set_current_dir(&temp_dir.path()).unwrap();
                WriteContext { temp_dir }
            }

            fn teardown(self) {
                self.temp_dir.close().unwrap();
            }
        }

        #[test_context::test_context(WriteContext)]
        #[test]
        #[serial_test::serial]
        fn custom_filename(ctx: &mut WriteContext) {
            let cur_dir = ctx.temp_dir.path();
            let configuration = Configuration {
                note_directory: str!(cur_dir, PathBuf),
                ..Default::default()
            };
            let note = Note::from("sample note").format(&configuration.note_template);

            match Markdown::from(NoteFile::custom_target("sample.md", &configuration)).write(&note)
            {
                Ok(res) => {
                    let expected = cur_dir.join("sample.md");
                    assert_file_exists!(res);
                    assert_eq!(expected, res);
                }
                Err(_) => panic!("write note month failed"),
            };

            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
        }

        #[test_context::test_context(WriteContext)]
        #[test]
        #[serial_test::serial]
        fn daily_rolling(ctx: &mut WriteContext) {
            let cur_dir = ctx.temp_dir.path();
            let configuration = Configuration {
                file_rolling: crate::noted::FileRolling::Daily,
                note_directory: str!(cur_dir, PathBuf),
                ..Default::default()
            };
            let note = Note::from("sample note").format(&configuration.note_template);

            match Markdown::from(NoteFile::target(&configuration)).write(&note) {
                Ok(_) => {
                    let now = chrono::Local::now();
                    let file_name = format!("{}.md", &now.format("%Y-%m-%d"));
                    let expected = cur_dir.join(file_name);
                    assert_file_exists!(expected);
                }
                Err(_) => panic!("write note month failed"),
            };
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
        }

        #[test_context::test_context(WriteContext)]
        #[test]
        #[serial_test::serial]
        fn month_rolling(ctx: &mut WriteContext) {
            let cur_dir = ctx.temp_dir.path();
            let configuration = Configuration {
                file_rolling: crate::noted::FileRolling::Month,
                note_directory: str!(cur_dir, PathBuf),
                ..Default::default()
            };
            let note = Note::from("sample note").format(&configuration.note_template);

            match Markdown::from(NoteFile::target(&configuration)).write(&note) {
                Ok(_) => {
                    let now = chrono::Local::now();
                    let file_name = format!("{}.md", &now.format("%Y-%m"));
                    let expected = cur_dir.join(file_name);
                    assert_file_exists!(expected);
                }
                Err(_) => panic!("write note month failed"),
            };
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
        }

        #[test_context::test_context(WriteContext)]
        #[test]
        #[serial_test::serial]
        fn week_rolling(ctx: &mut WriteContext) {
            let cur_dir = ctx.temp_dir.path();
            let configuration = Configuration {
                file_rolling: crate::noted::FileRolling::Week,
                note_directory: str!(cur_dir, PathBuf),
                ..Default::default()
            };
            let note = Note::from("sample note").format(&configuration.note_template);

            if Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok()
            {
                let now = chrono::Local::now();
                let file_name = format!("{}.md", &now.format("%Y-%W"));
                let expected = cur_dir.join(file_name);
                assert_file_exists!(expected);
            } else {
                panic!("write note month failed");
            };
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
        }

        #[test_context::test_context(WriteContext)]
        #[test]
        #[serial_test::serial]
        fn year_rolling(ctx: &mut WriteContext) {
            let cur_dir = ctx.temp_dir.path();
            let configuration = Configuration {
                file_rolling: crate::noted::FileRolling::Year,
                note_directory: str!(cur_dir, PathBuf),
                ..Default::default()
            };
            let note = Note::from("sample note").format(&configuration.note_template);

            if Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok()
            {
                let now = chrono::Local::now();
                let file_name = format!("{}.md", &now.format("%Y"));
                let expected = cur_dir.join(file_name);
                assert_file_exists!(expected);
            } else {
                panic!("write note month failed")
            };
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());
        }

        #[test_context::test_context(WriteContext)]
        #[test]
        #[serial_test::serial]
        fn note_strucutre(ctx: &mut WriteContext) {
            let cur_dir = ctx.temp_dir.path();
            let note_template = str!(indoc! {
                "%note%

                %tags%"
            });
            let configuration = Configuration {
                file_rolling: crate::noted::FileRolling::Year,
                note_directory: str!(cur_dir, PathBuf),
                note_template: NoteTemplate {
                    template: note_template,
                    ..Default::default()
                },
                ..Default::default()
            };
            let file = NoteFile::target(&configuration);

            let note = Note::from("sample note");
            assert!(Markdown::from(&file)
                .write(&note.format(&configuration.note_template))
                .is_ok());

            let note2 = Note::from(["sample note 2", "tag"].to_vec());
            assert!(Markdown::from(&file)
                .write(&note2.format(&configuration.note_template))
                .is_ok());

            let note3 = Note::from("sample note 3");
            assert!(Markdown::from(&file)
                .write(&note3.format(&configuration.note_template))
                .is_ok());

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
        }

        #[test_context::test_context(WriteContext)]
        #[test]
        #[serial_test::serial]
        fn mixed_rolling(ctx: &mut WriteContext) {
            let cur_dir = ctx.temp_dir.path();
            // Write Daily note.
            let mut configuration = Configuration {
                file_rolling: crate::noted::FileRolling::Daily,
                note_directory: str!(cur_dir, PathBuf),
                ..Default::default()
            };
            let note = Note::from("sample note").format(&configuration.note_template);

            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());

            // Change to week and write note
            configuration.file_rolling = crate::noted::FileRolling::Week;
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());

            // Change to never and write note
            configuration.file_rolling = crate::noted::FileRolling::Never;
            assert!(Markdown::from(NoteFile::target(&configuration))
                .write(&note)
                .is_ok());

            let options = glob::MatchOptions {
                case_sensitive: false,
                ..Default::default()
            };
            assert!(glob::glob_with("*", options).unwrap().count() == 3);

            let expected = cur_dir.join(NOTES_FILE_NAME);
            assert!(expected.exists());
        }
    }
}

#[cfg(test)]
mod notefile_tests {

    mod target_tests {
        use crate::noted::{Configuration, NoteFile};
        use crate::str;

        struct TargetContext {
            temp_dir: tempfile::TempDir,
            sub_dir: std::path::PathBuf,
        }

        impl test_context::TestContext for TargetContext {
            fn setup() -> TargetContext {
                let temp_dir = tempfile::tempdir().unwrap();
                let cur_dir = &temp_dir.path();

                std::env::set_current_dir(&cur_dir).unwrap();
                git2::Repository::init(&cur_dir).unwrap();
                let sub_dir = cur_dir.join("src").join("module");
                std::fs::create_dir_all(&sub_dir).unwrap();
                // wait!("5");
                TargetContext { temp_dir, sub_dir }
            }

            fn teardown(self) {
                self.temp_dir.close().unwrap();
            }
        }

        #[test]
        #[serial_test::serial]
        fn repository_specific_no_repository() {
            let temp_dir = tempfile::tempdir().unwrap();
            let cur_dir = temp_dir.path();
            std::env::set_current_dir(&cur_dir).unwrap();
            let configuration = Configuration {
                use_repository_specific: true,
                ..Default::default()
            };
            let target = NoteFile::target(&configuration);
            assert_ne!(
                format!("{}/notes.md", cur_dir.to_str().unwrap()),
                str!(target, PathBuf)
            );
            assert!(str!(target, PathBuf).starts_with("/home"));
        }

        #[test_context::test_context(TargetContext)]
        #[test]
        #[serial_test::serial]
        fn repository_specific_at_root(ctx: &mut TargetContext) {
            let cur_dir = ctx.temp_dir.path();
            let configuration = Configuration {
                use_repository_specific: true,
                ..Default::default()
            };
            let target = NoteFile::target(&configuration);
            assert_eq!(
                format!("{}/notes.md", cur_dir.to_str().unwrap()),
                str!(target, PathBuf)
            );
        }

        #[test_context::test_context(TargetContext)]
        #[test]
        #[serial_test::serial]
        fn repository_specific_at_subfolder(ctx: &mut TargetContext) {
            let cur_dir = ctx.sub_dir.clone();
            std::env::set_current_dir(&cur_dir).unwrap();

            let configuration = Configuration {
                use_repository_specific: true,
                ..Default::default()
            };
            let target = NoteFile::target(&configuration);
            assert_eq!(
                format!("{}/notes.md", ctx.temp_dir.path().to_str().unwrap()),
                str!(target, PathBuf)
            );
        }
    }

    mod taget_by_pattern_tests {
        use crate::noted::{Configuration, NoteFile};
        use crate::str;

        struct TargetByPatternContext {
            temp_dir: tempfile::TempDir,
        }

        impl test_context::TestContext for TargetByPatternContext {
            fn setup() -> TargetByPatternContext {
                let temp_dir = tempfile::tempdir().unwrap();

                std::env::set_current_dir(&temp_dir.path()).unwrap();
                safe_file_create!("2021.md");
                safe_file_create!("2021-01.md");
                safe_file_create!("2021-02.md");
                safe_file_create!("2021-04.md");
                safe_file_create!("2021-04-02.md");
                safe_file_create!("2021-04-03.md");
                TargetByPatternContext { temp_dir }
            }

            fn teardown(self) {
                self.temp_dir.close().unwrap();
            }
        }

        #[test_context::test_context(TargetByPatternContext)]
        #[test]
        #[serial_test::serial]
        fn empty_argument_returns_not_found_error(ctx: &mut TargetByPatternContext) {
            // ARAMGE
            let cur_dir = ctx.temp_dir.path();
            let configuration = Configuration {
                file_rolling: crate::noted::FileRolling::Year,
                note_directory: str!(cur_dir, PathBuf),
                ..Default::default()
            };
            // ACT
            match NoteFile::target_by_pattern(
                "",
                &std::path::PathBuf::from(&configuration.note_directory),
            ) {
                Ok(_) => panic!("no argument should return error"),
                Err(err) => assert_eq!(std::io::ErrorKind::Other, err.kind()),
            };
        }

        #[test_context::test_context(TargetByPatternContext)]
        #[test]
        #[serial_test::serial]
        fn no_matching_file_is_found_returns_none(ctx: &mut TargetByPatternContext) {
            // ARAMGE
            let cur_dir = ctx.temp_dir.path();
            let configuration = Configuration {
                file_rolling: crate::noted::FileRolling::Year,
                note_directory: str!(cur_dir, PathBuf),
                ..Default::default()
            };
            // ACT
            match NoteFile::target_by_pattern(
                "05",
                &std::path::PathBuf::from(&configuration.note_directory),
            ) {
                Ok(_) => panic!("pattern not found should return error"),
                Err(err) => assert_eq!(std::io::ErrorKind::NotFound, err.kind()),
            };
        }

        #[test_context::test_context(TargetByPatternContext)]
        #[test]
        #[serial_test::serial]
        fn find_by_filename(ctx: &mut TargetByPatternContext) {
            // ARAMGE
            let cur_dir = ctx.temp_dir.path();
            let configuration = Configuration {
                file_rolling: crate::noted::FileRolling::Year,
                note_directory: str!(cur_dir, PathBuf),
                ..Default::default()
            };
            // ACT
            match NoteFile::first_target_by_pattern(
                "2021.md",
                &std::path::PathBuf::from(&configuration.note_directory),
            ) {
                Ok(res) => assert_eq!(
                    format!("{}/{}", str!(cur_dir, PathBuf), "2021.md"),
                    str!(res, PathBuf)
                ),
                Err(err) => panic!("should find by filename: {}", err),
            }
        }

        #[test_context::test_context(TargetByPatternContext)]
        #[test]
        #[serial_test::serial]
        fn find_by_filename_wildcard_single_file(ctx: &mut TargetByPatternContext) {
            // ARAMGE
            let cur_dir = ctx.temp_dir.path();
            let configuration = Configuration {
                note_directory: str!(cur_dir, PathBuf),
                ..Default::default()
            };
            // ACT
            match NoteFile::first_target_by_pattern(
                "*21.md",
                &std::path::PathBuf::from(&configuration.note_directory),
            ) {
                Ok(res) => assert_eq!(
                    format!("{}/{}", str!(cur_dir, PathBuf), "2021.md"),
                    str!(res, PathBuf)
                ),
                Err(err) => panic!("should return single file: {}", err),
            }
        }

        #[test_context::test_context(TargetByPatternContext)]
        #[test]
        #[serial_test::serial]
        fn find_by_filename_wildcard_no_char_before_ext(ctx: &mut TargetByPatternContext) {
            // ARAMGE
            let cur_dir = ctx.temp_dir.path();
            let configuration = Configuration {
                file_rolling: crate::noted::FileRolling::Daily,
                note_directory: str!(cur_dir, PathBuf),
                ..Default::default()
            };
            // ACT
            match NoteFile::first_target_by_pattern(
                "2021-01*",
                &std::path::PathBuf::from(&configuration.note_directory),
            ) {
                Ok(res) => assert_eq!(
                    format!("{}/{}", str!(cur_dir, PathBuf), "2021-01.md"),
                    str!(res, PathBuf)
                ),
                Err(err) => panic!("should find with wilcard extension: {}", err),
            };
        }

        #[test_context::test_context(TargetByPatternContext)]
        #[test]
        #[serial_test::serial]
        fn find_by_filename_wildcard_multiple_files_rolling_mix(ctx: &mut TargetByPatternContext) {
            // ARAMGE
            let cur_dir = ctx.temp_dir.path();
            let configuration = Configuration {
                file_rolling: crate::noted::FileRolling::Daily,
                note_directory: str!(cur_dir, PathBuf),
                ..Default::default()
            };
            // ACT
            match NoteFile::first_target_by_pattern(
                "2021-04*",
                &std::path::PathBuf::from(&configuration.note_directory),
            ) {
                Ok(res) => assert_eq!(
                    format!("{}/{}", str!(cur_dir, PathBuf), "2021-04-02.md"),
                    str!(res, PathBuf)
                ),
                Err(err) => panic!("should return first file: {}", err),
            }
        }

        #[test_context::test_context(TargetByPatternContext)]
        #[test]
        #[serial_test::serial]
        fn find_by_filename_wildcard_multiple_files_sorting(ctx: &mut TargetByPatternContext) {
            // ARAMGE
            let cur_dir = ctx.temp_dir.path();
            let configuration = Configuration {
                file_rolling: crate::noted::FileRolling::Daily,
                note_directory: str!(cur_dir, PathBuf),
                ..Default::default()
            };
            // ACT
            match NoteFile::first_target_by_pattern(
                "2021-0*",
                &std::path::PathBuf::from(&configuration.note_directory),
            ) {
                Ok(res) => assert_eq!(
                    format!("{}/{}", str!(cur_dir, PathBuf), "2021-01.md"),
                    str!(res, PathBuf)
                ),
                Err(err) => panic!(
                    "should return first file based on filesystem sorting: {}",
                    err
                ),
            };
        }

        #[test_context::test_context(TargetByPatternContext)]
        #[test]
        #[serial_test::serial]
        fn find_by_filename_wildcard_multiple_files(ctx: &mut TargetByPatternContext) {
            // ARAMGE
            let cur_dir = ctx.temp_dir.path();
            let configuration = Configuration {
                file_rolling: crate::noted::FileRolling::Daily,
                note_directory: str!(cur_dir, PathBuf),
                ..Default::default()
            };
            // ACT
            match NoteFile::target_by_pattern(
                "2021-04*",
                &std::path::PathBuf::from(&configuration.note_directory),
            ) {
                Ok(res) => assert_eq!(3, res.len()),
                Err(err) => panic!(
                    "should return 3 files based: {:?}",
                    err
                ),
            };
        }
    }
}

#[cfg(test)]
mod command_test {
    use crate::noted::{Command, Configuration, FileRolling, NoteTemplate, NOTES_FILE_NAME};
    use crate::{str, vec_of_strings};
    use indoc::indoc;

    struct CommandContext {
        configuration: Configuration,
        temp_dir: tempfile::TempDir,
    }

    impl test_context::TestContext for CommandContext {
        fn setup() -> CommandContext {
            let temp_dir = tempfile::tempdir().unwrap();
            std::env::set_current_dir(&temp_dir.path()).unwrap();

            CommandContext {
                configuration: Configuration {
                    note_directory: str!(temp_dir.path(), PathBuf),
                    use_repository_specific: false,
                    file_rolling: FileRolling::Never,
                    template_file: "".to_string(),
                    note_template: NoteTemplate {
                        template: str!(indoc! {
                            "%note%

                            %tags%"
                        }),
                        ..Default::default()
                    },
                },
                temp_dir,
            }
        }

        fn teardown(self) {
            self.temp_dir.close().unwrap();
        }
    }

    #[test_context::test_context(CommandContext)]
    #[test]
    #[serial_test::serial]
    fn take_note(ctx: &mut CommandContext) {
        // ARANGE
        let command = Command::Note {
            open_after_write: false,
            note: "Sample Note".to_string(),
            tags: Vec::new(),
        };
        // ACT
        match command.invoke(Some(ctx.configuration.clone())) {
            Ok(_) => {
                // ASSERT
                let expected_file = std::path::PathBuf::from(&ctx.configuration.note_directory)
                    .join(NOTES_FILE_NAME);
                assert_file_exists!(expected_file);
                let expected_note_content = vec_of_strings!("Sample Note", "", "---");

                let read_note_file = std::fs::File::open(expected_file).unwrap();
                let raw_note: Vec<String> =
                    std::io::BufRead::lines(std::io::BufReader::new(read_note_file))
                        .map(|l| l.unwrap())
                        .collect();

                for i in 0..raw_note.len() - 1 {
                    assert_eq!(expected_note_content[i], raw_note[i]);
                }
            }
            Err(err) => panic!("Command Note Error: {:?}", err),
        };
    }

    #[test_context::test_context(CommandContext)]
    #[test]
    #[serial_test::serial]
    fn take_note_with_tags(ctx: &mut CommandContext) {
        // ARANGE
        let command = Command::Note {
            open_after_write: false,
            note: "Sample Note".to_string(),
            tags: vec_of_strings!("sample", "test"),
        };
        // ACT
        match command.invoke(Some(ctx.configuration.clone())) {
            Ok(_) => {
                // ASSERT
                let expected_file = std::path::PathBuf::from(&ctx.configuration.note_directory)
                    .join(NOTES_FILE_NAME);
                assert_file_exists!(expected_file);
                let expected_note_content =
                    vec_of_strings!("Sample Note", "", "#sample;#test", "", "---");

                let read_note_file = std::fs::File::open(expected_file).unwrap();
                let raw_note: Vec<String> =
                    std::io::BufRead::lines(std::io::BufReader::new(read_note_file))
                        .map(|l| l.unwrap())
                        .collect();

                for i in 0..raw_note.len() - 1 {
                    assert_eq!(expected_note_content[i], raw_note[i]);
                }
            }
            Err(err) => panic!("Command Note Error: {:?}", err),
        };
    }

    #[test_context::test_context(CommandContext)]
    #[test]
    #[serial_test::serial]
    fn take_note_open(ctx: &mut CommandContext) {
        // ARANGE
        let command = Command::Note {
            open_after_write: true,
            note: "Sample Note".to_string(),
            tags: Vec::new(),
        };
        // ACT
        match command.invoke(Some(ctx.configuration.clone())) {
            Ok(res) => {
                // ASSERT
                let expected_file = std::path::PathBuf::from(&ctx.configuration.note_directory)
                    .join(NOTES_FILE_NAME);
                assert_file_exists!(expected_file);
                assert_eq!(expected_file, res.unwrap());
            }
            Err(err) => panic!("Command Note and open Error: {:?}", err),
        };
    }

    #[test_context::test_context(CommandContext)]
    #[test]
    #[serial_test::serial]
    fn create_note(ctx: &mut CommandContext) {
        // ARANGE
        let filename = "my-custom-file".to_string();
        let command = Command::Create {
            filename: filename.clone(),
        };
        // ACT
        match command.invoke(Some(ctx.configuration.clone())) {
            Ok(res) => {
                // ASSERT
                let expected_file = std::path::PathBuf::from(&ctx.configuration.note_directory)
                    .join(format!("{}.md", filename));
                assert_file_exists!(expected_file);
                assert_eq!(expected_file, res.unwrap());
            }
            Err(err) => panic!("Command Create Error: {:?}", err),
        };
    }

    #[test_context::test_context(CommandContext)]
    #[test]
    #[serial_test::serial]
    fn open_note(ctx: &mut CommandContext) {
        // ARANGE
        // Create a note
        Command::Note {
            open_after_write: false,
            note: "Sample Note".to_string(),
            tags: Vec::new(),
        }
        .invoke(Some(ctx.configuration.clone()))
        .unwrap();
        let command = Command::Open { filename: None };
        // ACT
        match command.invoke(Some(ctx.configuration.clone())) {
            Ok(res) => {
                // ASSERT
                let expected_file = std::path::PathBuf::from(&ctx.configuration.note_directory)
                    .join(NOTES_FILE_NAME);
                assert_file_exists!(expected_file);
                assert_eq!(expected_file, res.unwrap());
            }
            Err(err) => panic!("Command Open Error: {:?}", err),
        };
    }

    #[test_context::test_context(CommandContext)]
    #[test]
    #[serial_test::serial]
    fn open_note_custom_file(ctx: &mut CommandContext) {
        // ARANGE
        let filename = "my-custom-file".to_string();
        safe_file_create!(&filename);
        let command = Command::Open {
            filename: Some(filename.clone()),
        };
        // ACT
        match command.invoke(Some(ctx.configuration.clone())) {
            Ok(res) => {
                // ASSERT
                let expected_file =
                    std::path::PathBuf::from(&ctx.configuration.note_directory).join(filename);
                assert_file_exists!(expected_file);
                assert_eq!(expected_file, res.unwrap());
            }
            Err(err) => panic!("Command Open custom Error: {:?}", err),
        };
    }

    #[test_context::test_context(CommandContext)]
    #[test]
    #[serial_test::serial]
    fn config(ctx: &mut CommandContext) {
        // ARANGE
        let command = Command::Config;
        // ACT
        match command.invoke(Some(ctx.configuration.clone())) {
            Ok(res) => {
                // ASSERT
                let expected_config_path = Configuration::file_path();
                let config_path = res.unwrap();
                assert!(str!(config_path, PathBuf).ends_with(".config"));
                assert_eq!(expected_config_path, config_path);
            }
            Err(err) => panic!("Command Config Error: {:?}", err),
        };
    }

    #[test_context::test_context(CommandContext)]
    #[test]
    #[serial_test::serial]
    fn search(ctx: &mut CommandContext) {
        // ARANGE
        // Create a note
        Command::Note {
            open_after_write: false,
            note: "Sample Note".to_string(),
            tags: Vec::new(),
        }
        .invoke(Some(ctx.configuration.clone()))
        .unwrap();
        let command = Command::Search {
            tag: false,
            pattern: ".?ample.*".to_string(),
            file_pattern: None,
            output_to_file: true,
        };
        // ACT
        match command.invoke(Some(ctx.configuration.clone())) {
            Ok(_) => {
                // ASSERT
                let search_log = std::path::PathBuf::from("search_result.txt");
                assert_file_exists!(search_log);
                let expected_content = vec_of_strings!(
                    "File                           | Line | Content",
                    format!(
                        "{}/notes.md       | 1    | Sample Note",
                        str!(ctx.temp_dir.path(), PathBuf)
                    )
                );

                let read_file = std::fs::File::open(search_log).unwrap();
                let raw: Vec<String> = std::io::BufRead::lines(std::io::BufReader::new(read_file))
                    .map(|l| l.unwrap())
                    .collect();

                for i in 0..raw.len() - 1 {
                    assert_eq!(expected_content[i], raw[i]);
                }
            }
            Err(err) => panic!("Command Search Error: {:?}", err),
        };
    }
}
