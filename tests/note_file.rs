use noted::{configuration::Configuration, file_rolling::FileRolling, note_file::NoteFile, str, NOTES_FILE_NAME, safe_file_create};

#[test]
fn search_invalid_pattern() {
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
from_tests!(from_daily, FileRolling::Daily, "%Y-%m-%d");
from_tests!(from_month, FileRolling::Month, "%Y-%m");
from_tests!(from_week, FileRolling::Week, "%Y-%W");
from_tests!(from_year, FileRolling::Year, "%Y");
from_tests!(from_never, FileRolling::Never, "notes");

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

custom_target_tests!(custom_target_file_with_md_extension, "test.md", "test.md");
custom_target_tests!(custom_target_file_without_md_extension, "test", "test.md");
custom_target_tests!(
    custom_target_file_with_other_extension,
    "test.ini",
    "test.ini.md"
);
custom_target_tests!(
    custom_target_file_without_name_returns_from_config,
    "",
    NOTES_FILE_NAME
);

#[test]
fn custom_target_repository_specific_returns_note_directory() {
    let configuration = Configuration {
        use_repository_specific: true,
        file_rolling: FileRolling::Never,
        ..Default::default()
    };
    let target = NoteFile::custom_target("sample_not_inside_repo", &configuration);
    assert_eq!(
        std::path::PathBuf::from(configuration.note_directory).join("sample_not_inside_repo.md"),
        target
    );
}

struct RepositoryTargetContext {
    temp_dir: tempfile::TempDir,
    sub_dir: std::path::PathBuf,
}

impl test_context::TestContext for RepositoryTargetContext {
    fn setup() -> RepositoryTargetContext {
        let temp_dir = tempfile::tempdir().unwrap();
        let cur_dir = &temp_dir.path();

        std::env::set_current_dir(&cur_dir).unwrap();
        git2::Repository::init(&cur_dir).unwrap();
        let sub_dir = cur_dir.join("src").join("module");
        std::fs::create_dir_all(&sub_dir).unwrap();
        // wait!("5");
        RepositoryTargetContext { temp_dir, sub_dir }
    }

    fn teardown(self) {
        self.temp_dir.close().unwrap();
    }
}

#[test]
#[serial_test::serial]
fn repository_specific_target_no_repository() {
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

#[test_context::test_context(RepositoryTargetContext)]
#[test]
#[serial_test::serial]
fn repository_specific_target_at_root(ctx: &mut RepositoryTargetContext) {
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

#[test_context::test_context(RepositoryTargetContext)]
#[test]
#[serial_test::serial]
fn repository_specific_target_at_subfolder(ctx: &mut RepositoryTargetContext) {
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

#[test]
fn user_directory_target_with_file_rolling_month() {
    let configuration = Configuration {
        file_rolling: FileRolling::Month,
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
fn invalid_pattern_for_target_error_bubble_up() {
    match NoteFile::target_by_pattern("[", tempfile::tempdir().unwrap().path()) {
        Ok(_) => panic!("should not be ok"),
        Err(err) => {
            assert_eq!(std::io::ErrorKind::Other, err.kind());
        }
    }
}

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
fn empty_argument_for_target_pattern_returns_not_found_error(ctx: &mut TargetByPatternContext) {
    // ARAMGE
    let cur_dir = ctx.temp_dir.path();
    let configuration = Configuration {
        file_rolling: FileRolling::Year,
        note_directory: str!(cur_dir, PathBuf),
        ..Default::default()
    };
    // ACT
    match NoteFile::target_by_pattern("", &std::path::PathBuf::from(&configuration.note_directory))
    {
        Ok(_) => panic!("no argument should return error"),
        Err(err) => assert_eq!(std::io::ErrorKind::Other, err.kind()),
    };
}

#[test_context::test_context(TargetByPatternContext)]
#[test]
#[serial_test::serial]
fn no_matching_file_by_target_pattern_found_returns_none(ctx: &mut TargetByPatternContext) {
    // ARAMGE
    let cur_dir = ctx.temp_dir.path();
    let configuration = Configuration {
        file_rolling: FileRolling::Year,
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
fn target_by_exact_filename(ctx: &mut TargetByPatternContext) {
    // ARAMGE
    let cur_dir = ctx.temp_dir.path();
    let configuration = Configuration {
        file_rolling: FileRolling::Year,
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
fn target_by_filename_wildcard_single_file(ctx: &mut TargetByPatternContext) {
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
fn target_by_filename_wildcard_no_char_before_ext(ctx: &mut TargetByPatternContext) {
    // ARAMGE
    let cur_dir = ctx.temp_dir.path();
    let configuration = Configuration {
        file_rolling: FileRolling::Daily,
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
fn target_by_filename_wildcard_multiple_files_rolling_mix(ctx: &mut TargetByPatternContext) {
    // ARAMGE
    let cur_dir = ctx.temp_dir.path();
    let configuration = Configuration {
        file_rolling: FileRolling::Daily,
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
fn target_by_filename_wildcard_multiple_files_sorting(ctx: &mut TargetByPatternContext) {
    // ARAMGE
    let cur_dir = ctx.temp_dir.path();
    let configuration = Configuration {
        file_rolling: FileRolling::Daily,
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
fn target_by_filename_wildcard_multiple_files(ctx: &mut TargetByPatternContext) {
    // ARAMGE
    let cur_dir = ctx.temp_dir.path();
    let configuration = Configuration {
        file_rolling: FileRolling::Daily,
        note_directory: str!(cur_dir, PathBuf),
        ..Default::default()
    };
    // ACT
    match NoteFile::target_by_pattern(
        "2021-04*",
        &std::path::PathBuf::from(&configuration.note_directory),
    ) {
        Ok(res) => assert_eq!(3, res.len()),
        Err(err) => panic!("should return 3 files based: {:?}", err),
    };
}
