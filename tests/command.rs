use std::{path, io, fs, env};

use indoc::indoc;
use noted::{
    command::Command, configuration::Configuration, file_rolling::FileRolling,
    note_template::NoteTemplate, str, vec_of_strings, NOTES_FILE_NAME, assert_file_exists, safe_file_create,
};

struct CommandContext {
    configuration: Configuration,
    temp_dir: tempfile::TempDir,
}

impl test_context::TestContext for CommandContext {
    fn setup() -> CommandContext {
        let temp_dir = tempfile::tempdir().unwrap();
        env::set_current_dir(&temp_dir.path()).unwrap();

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
            let expected_file =
                path::PathBuf::from(&ctx.configuration.note_directory).join(NOTES_FILE_NAME);
            assert_file_exists!(expected_file);
            let expected_note_content = vec_of_strings!("Sample Note", "", "---");

            let read_note_file = fs::File::open(expected_file).unwrap();
            let raw_note: Vec<String> =
                io::BufRead::lines(io::BufReader::new(read_note_file))
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
            let expected_file =
                path::PathBuf::from(&ctx.configuration.note_directory).join(NOTES_FILE_NAME);
            assert_file_exists!(expected_file);
            let expected_note_content =
                vec_of_strings!("Sample Note", "", "#sample;#test", "", "---");

            let read_note_file = fs::File::open(expected_file).unwrap();
            let raw_note: Vec<String> =
                io::BufRead::lines(io::BufReader::new(read_note_file))
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
fn take_note_and_open(ctx: &mut CommandContext) {
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
            let expected_file =
                path::PathBuf::from(&ctx.configuration.note_directory).join(NOTES_FILE_NAME);
            assert_file_exists!(expected_file);
            assert_eq!(expected_file, res.unwrap());
        }
        Err(err) => panic!("Command Note and open Error: {:?}", err),
    };
}

#[test_context::test_context(CommandContext)]
#[test]
#[serial_test::serial]
fn create_note_command(ctx: &mut CommandContext) {
    // ARANGE
    let filename = "my-custom-file".to_string();
    let command = Command::Create {
        filename: filename.clone(),
    };
    // ACT
    match command.invoke(Some(ctx.configuration.clone())) {
        Ok(res) => {
            // ASSERT
            let expected_file = path::PathBuf::from(&ctx.configuration.note_directory)
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
fn open_note_command(ctx: &mut CommandContext) {
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
            let expected_file =
                path::PathBuf::from(&ctx.configuration.note_directory).join(NOTES_FILE_NAME);
            assert_file_exists!(expected_file);
            assert_eq!(expected_file, res.unwrap());
        }
        Err(err) => panic!("Command Open Error: {:?}", err),
    };
}

#[test_context::test_context(CommandContext)]
#[test]
#[serial_test::serial]
fn open_note_command_custom_file(ctx: &mut CommandContext) {
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
                path::PathBuf::from(&ctx.configuration.note_directory).join(filename);
            assert_file_exists!(expected_file);
            assert_eq!(expected_file, res.unwrap());
        }
        Err(err) => panic!("Command Open custom Error: {:?}", err),
    };
}

#[test_context::test_context(CommandContext)]
#[test]
#[serial_test::serial]
fn config_command(ctx: &mut CommandContext) {
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
fn search_command(ctx: &mut CommandContext) {
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
            let search_log = path::PathBuf::from("search_result.txt");
            assert_file_exists!(search_log);
            let expected_content = vec_of_strings!(
                "File                           | Line | Content",
                format!(
                    "{}/notes.md       | 1    | Sample Note",
                    str!(ctx.temp_dir.path(), PathBuf)
                )
            );

            let read_file = fs::File::open(search_log).unwrap();
            let raw: Vec<String> = io::BufRead::lines(io::BufReader::new(read_file))
                .map(|l| l.unwrap())
                .collect();

            for i in 0..raw.len() - 1 {
                assert_eq!(expected_content[i], raw[i]);
            }
        }
        Err(err) => panic!("Command Search Error: {:?}", err),
    };
}
