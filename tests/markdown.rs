use indoc::indoc;
use noted::{
    configuration::Configuration, file_rolling::FileRolling, markdown::Markdown, note::Note,
    note_file::NoteFile, note_template::NoteTemplate, str, vec_of_strings, SearchArguments,
    NOTES_FILE_NAME, assert_file_exists,
};

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
fn search_yields_single_result_in_one_file(ctx: &mut SearchContext) {
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
fn search_yields_multiple_results_in_one_file(ctx: &mut SearchContext) {
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
fn search_yields_single_result_in_multiple_files(ctx: &mut SearchContext) {
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
fn search_yields_multiple_results_in_multiple_files(ctx: &mut SearchContext) {
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
fn search_for_tags(ctx: &mut SearchContext) {
    // ARRANGE
    let cur_dir = std::env::current_dir().unwrap();
    let mut configuration = Configuration {
        note_directory: str!(cur_dir, PathBuf),
        file_rolling: FileRolling::Never,
        ..Default::default()
    };
    let note =
        Note::from(["sample note", "test", "test1"].to_vec()).format(&configuration.note_template);
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
fn search_finds_multiple_occurence_in_filtered_files(ctx: &mut SearchContext) {
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
fn search_yields_no_results(ctx: &mut SearchContext) {
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
fn search_output_focus_on_occurence(ctx: &mut SearchContext) {
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
fn search_output_focus_on_occurence_with_long_line(ctx: &mut SearchContext) {
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
fn write_with_custom_filename(ctx: &mut WriteContext) {
    let cur_dir = ctx.temp_dir.path();
    let configuration = Configuration {
        note_directory: str!(cur_dir, PathBuf),
        ..Default::default()
    };
    let note = Note::from("sample note").format(&configuration.note_template);

    match Markdown::from(NoteFile::custom_target("sample.md", &configuration)).write(&note) {
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
fn write_with_daily_rolling(ctx: &mut WriteContext) {
    let cur_dir = ctx.temp_dir.path();
    let configuration = Configuration {
        file_rolling: FileRolling::Daily,
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
fn write_with_month_rolling(ctx: &mut WriteContext) {
    let cur_dir = ctx.temp_dir.path();
    let configuration = Configuration {
        file_rolling: FileRolling::Month,
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
fn write_with_week_rolling(ctx: &mut WriteContext) {
    let cur_dir = ctx.temp_dir.path();
    let configuration = Configuration {
        file_rolling: FileRolling::Week,
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
fn write_with_year_rolling(ctx: &mut WriteContext) {
    let cur_dir = ctx.temp_dir.path();
    let configuration = Configuration {
        file_rolling: FileRolling::Year,
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
        file_rolling: FileRolling::Year,
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
    let raw_note: Vec<String> = std::io::BufRead::lines(std::io::BufReader::new(read_note_file))
        .map(|l| l.unwrap())
        .collect();

    for i in 0..raw_note.len() - 1 {
        assert_eq!(expected_note_content[i], raw_note[i]);
    }
}

#[test_context::test_context(WriteContext)]
#[test]
#[serial_test::serial]
fn write_with_mixed_rolling(ctx: &mut WriteContext) {
    let cur_dir = ctx.temp_dir.path();
    // Write Daily note.
    let mut configuration = Configuration {
        file_rolling: FileRolling::Daily,
        note_directory: str!(cur_dir, PathBuf),
        ..Default::default()
    };
    let note = Note::from("sample note").format(&configuration.note_template);

    assert!(Markdown::from(NoteFile::target(&configuration))
        .write(&note)
        .is_ok());

    // Change to week and write note
    configuration.file_rolling = FileRolling::Week;
    assert!(Markdown::from(NoteFile::target(&configuration))
        .write(&note)
        .is_ok());

    // Change to never and write note
    configuration.file_rolling = FileRolling::Never;
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
