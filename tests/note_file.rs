use noted::{configuration::Configuration, file_rolling::FileRolling, note_file::NoteFile, str, safe_file_create};

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
fn when_repository_specific_is_and_target_is_no_repository_then_default_to_note_directory() {
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
    assert!(str!(target, PathBuf).starts_with("/"));
}

#[test_context::test_context(RepositoryTargetContext)]
#[test]
#[serial_test::serial]
fn when_repository_specific_is_used_then_note_at_root_is_created(ctx: &mut RepositoryTargetContext) {
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
fn when_repository_specific_is_used_inside_subfolder_then_note_at_root_is_created(ctx: &mut RepositoryTargetContext) {
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
fn when_empty_argument_for_target_pattern_is_used_then_returns_not_found_error(ctx: &mut TargetByPatternContext) {
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
fn when_no_matching_file_by_target_pattern_is_found_then_returns_none(ctx: &mut TargetByPatternContext) {
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
fn when_target_by_exact_filename_then_this_file_is_returned(ctx: &mut TargetByPatternContext) {
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
fn when_target_by_filename_wildcard_finds_single_file_then_single_file_is_returned(ctx: &mut TargetByPatternContext) {
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
fn when_target_by_filename_wildcard_no_char_before_ext_then_file_is_returned(ctx: &mut TargetByPatternContext) {
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
fn when_target_by_filename_wildcard_finds_multiple_files_rolling_mix_then_first_file_is_returned(ctx: &mut TargetByPatternContext) {
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
fn when_target_by_filename_wildcard_finds_multiple_files_then_sorting_is_applied(ctx: &mut TargetByPatternContext) {
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
fn when_target_by_filename_wildcard_multiple_files_then_all_files_are_returned(ctx: &mut TargetByPatternContext) {
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
