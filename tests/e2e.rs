use indoc::indoc;
#[allow(unused_imports)]
use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    process::Command,
};

#[allow(unused_macros)]
macro_rules! flush_output {
    ($d:expr, $f:expr) => {
        Write::write_all(
            &mut BufWriter::new(File::create($d.join("stdout.txt")).unwrap()),
            &$f.stdout,
        )
        .unwrap();
        Write::write_all(
            &mut BufWriter::new(File::create($d.join("stderr.txt")).unwrap()),
            &$f.stderr,
        )
        .unwrap();
    };
}

fn wait(time: &str) {
    let mut child = std::process::Command::new("sleep")
        .arg(time)
        .spawn()
        .unwrap();
    let _result = child.wait().unwrap();
}

use noted::{configuration::Configuration, file_rolling::FileRolling, note_template::NoteTemplate};
use serial_test::serial;

const EXECUTABLE: &str = "./debug/target/debug/noted";

fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

struct End2EndContext {
    temp_dir: tempfile::TempDir,
}

impl test_context::TestContext for End2EndContext {
    fn setup() -> End2EndContext {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut config = Configuration::new();
        config.file_rolling = FileRolling::Never;
        config.note_directory = temp_dir.path().to_str().unwrap().to_string();
        config.note_template = NoteTemplate {
            date_format: "%Y%m".to_string(),
            ..config.note_template
        };
        Configuration::save(&config);
        End2EndContext { temp_dir }
    }

    fn teardown(self) {
        self.temp_dir.close().unwrap();
        fs::remove_dir_all(Configuration::folder()).unwrap();
    }
}

#[test_context::test_context(End2EndContext)]
#[test]
#[serial]
fn when_note_with_tags_is_taken_then_a_note_file_is_created(ctx: &mut End2EndContext) {
    let now = chrono::Local::now();
    let timestamp = format!("{}", &now.format("%Y%m"));
    let expected = vec![
        timestamp,
        "".to_string(),
        "sample note".to_string(),
        "".to_string(),
        "---".to_string(),
    ];
    Command::new(EXECUTABLE)
        .args(["sample note"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    wait("3");
    let read_note = fs::File::open(ctx.temp_dir.path().join("notes.md")).unwrap();
    let actual: Vec<String> = io::BufRead::lines(io::BufReader::new(read_note))
        .map(|l| l.unwrap())
        .collect();

    for i in 0..actual.len() - 1 {
        assert_eq!(expected[i], actual[i], "on line {}", i);
    }
}

#[test_context::test_context(End2EndContext)]
#[test]
#[serial]
fn when_search_for_note_then_files_in_note_folder_are_searched(ctx: &mut End2EndContext) {
    let expected = format!(
        indoc!(
            "
        File              | Line | Content
        {}/notes.md       | 3    | sample note
        "
        ),
        &ctx.temp_dir.path().to_str().unwrap()
    );
    Command::new(EXECUTABLE)
        .args(["sample note"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    wait("3");
    let output = Command::new(EXECUTABLE)
        .args(["search", "sample"])
        .output()
        .unwrap();

    let str_stdout = std::str::from_utf8(&output.stdout).unwrap();
    let str_stderr = std::str::from_utf8(&output.stderr).unwrap();

    assert_eq!("", remove_whitespace(str_stderr));
    assert_eq!(remove_whitespace(&expected), remove_whitespace(str_stdout));
}

#[test_context::test_context(End2EndContext)]
#[test]
#[serial]
fn when_search_for_note_in_multiple_files_then_files_in_note_folder_are_searched(
    ctx: &mut End2EndContext,
) {
    Write::write_all(
        &mut BufWriter::new(File::create(&ctx.temp_dir.path().join("2021.md")).unwrap()),
        indoc!(
            "
        Sample Note 1

        #Tag

        ---
        Sample Note 2
        ---
        "
        )
        .as_bytes(),
    )
    .unwrap();
    Write::write_all(
        &mut BufWriter::new(File::create(&ctx.temp_dir.path().join("2021-02.md")).unwrap()),
        indoc!(
            "
        Sample Note 3

        #Tag2

        ---
        Sample Note 4
        ---
        "
        )
        .as_bytes(),
    )
    .unwrap();
    let expected = format!(
        indoc!(
            "
        File              | Line | Content
        {0}/2021-02.md    | 1    | Sample Note 3
        {0}/2021-02.md    | 6    | Sample Note 4
        {0}/2021.md       | 1    | Sample Note 1
        {0}/2021.md       | 6    | Sample Note 2
        "
        ),
        &ctx.temp_dir.path().to_str().unwrap()
    );
    wait("3");
    let output = Command::new(EXECUTABLE)
        .args(["search", "Sample"])
        .output()
        .unwrap();

    let str_stdout = std::str::from_utf8(&output.stdout).unwrap();
    let str_stderr = std::str::from_utf8(&output.stderr).unwrap();

    assert_eq!("", remove_whitespace(str_stderr));
    assert_eq!(remove_whitespace(&expected), remove_whitespace(str_stdout));
}

#[test_context::test_context(End2EndContext)]
#[test]
#[serial]
fn when_search_for_tag_wildcard_in_multiple_files_then_files_in_note_folder_are_searched(
    ctx: &mut End2EndContext,
) {
    Write::write_all(
        &mut BufWriter::new(File::create(&ctx.temp_dir.path().join("2021.md")).unwrap()),
        indoc!(
            "
        Sample Note 1

        #Tag

        ---
        Sample Note 2
        ---
        "
        )
        .as_bytes(),
    )
    .unwrap();
    Write::write_all(
        &mut BufWriter::new(File::create(&ctx.temp_dir.path().join("2021-02.md")).unwrap()),
        indoc!(
            "
        Sample Note 3

        #Tag2

        ---
        Sample Note 4
        ---
        "
        )
        .as_bytes(),
    )
    .unwrap();
    let expected = format!(
        indoc!(
            "
        File              | Line | Content
        {0}/2021-02.md    | 3    | #Tag2
        {0}/2021.md       | 3    | #Tag

        "
        ),
        &ctx.temp_dir.path().to_str().unwrap()
    );
    wait("3");
    let output = Command::new(EXECUTABLE)
        .args(["search", "-t", "Tag.?"])
        .output()
        .unwrap();

    let str_stdout = std::str::from_utf8(&output.stdout).unwrap();
    let str_stderr = std::str::from_utf8(&output.stderr).unwrap();

    assert_eq!("", remove_whitespace(str_stderr));
    assert_eq!(remove_whitespace(&expected), remove_whitespace(str_stdout));
}
