use std::{fs, io};
use noted::{configuration::Configuration, file_rolling::FileRolling};
use serial_test::serial;

const CONFIGURATION_FOLDER: &str = "noted";
const CONFIGURATION_FILE_NAME: &str = "noted.config";
const CONFIGURATION_TEMPLATE_FILE_NAME: &str = "noted.template";

#[test]
#[serial]
fn when_configuration_is_written_then_configuration_file_is_created_and_can_be_parsed() {
    let expected_config = vec![
        "NOTE_DIRECTORY=/home/".to_string(),
        "USE_REPOSITORY_SPECIFIC=false".to_string(),
        "FILE_ROLLING=Daily".to_string(),
        "DATE_FORMAT=%F %T".to_string(),
        "NOTE_TEMPLATE=/home/".to_string(),
    ];
    let expected_template = vec![
        "%date_format%".to_string(),
        "".to_string(),
        "%note%".to_string(),
        "".to_string(),
        "%tags%".to_string(),
    ];
    let config_dir = dirs::config_dir().unwrap().join(CONFIGURATION_FOLDER);
    let config_file = dirs::config_dir()
        .unwrap()
        .join(CONFIGURATION_FOLDER)
        .join(CONFIGURATION_FILE_NAME);
    let template_file = dirs::config_dir()
        .unwrap()
        .join(CONFIGURATION_FOLDER)
        .join(CONFIGURATION_TEMPLATE_FILE_NAME);
    let _r = fs::remove_dir_all(&config_dir);
    assert!(!&config_dir.exists());
    Configuration::new();
    // assert
    let read_config_file = fs::File::open(config_file).unwrap();
    let raw_config: Vec<String> = io::BufRead::lines(io::BufReader::new(read_config_file))
        .map(|l| l.unwrap())
        .collect();

    for i in 0..raw_config.len() - 1 {
        assert!(raw_config[i].starts_with(&expected_config[i].to_string()), "on line {}", i);
    }
    let read_template_file = fs::File::open(template_file).unwrap();
    let raw_template: Vec<String> = io::BufRead::lines(io::BufReader::new(read_template_file))
        .map(|l| l.unwrap())
        .collect();

    for i in 0..raw_template.len() - 1 {
        assert_eq!(expected_template[i], raw_template[i], "on line {}", i);
    }
    let removed = fs::remove_dir_all(&config_dir);
    assert!(removed.is_ok());
    assert!(!config_dir.exists());
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
    assert!(config.note_directory.starts_with("/home"));
    assert!(!config.use_repository_specific);
    assert_eq!(FileRolling::Daily, config.file_rolling);
    assert!(config.template_file.contains("noted/noted.template"));
    assert!(config.note_template.template.starts_with("%date_format%"));
    // clean up
    let removed = std::fs::remove_dir_all(Configuration::folder());
    assert!(removed.is_ok());
    assert!(!Configuration::folder().exists());
}