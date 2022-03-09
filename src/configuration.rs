use std::{
    env,
    fs::{create_dir_all, File},
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

use crate::{file_rolling::FileRolling, note_template::NoteTemplate, LINE_ENDING};
use indoc::indoc;

const CONFIGURATION_FOLDER: &str = "noted";
const CONFIGURATION_FILE_NAME: &str = "noted.config";

const CONFIGURATION_KEY_NOTE_DIRECTORY: &str = "NOTE_DIRECTORY";
const CONFIGURATION_KEY_USE_REPOSITORY_SPECIFIC: &str = "USE_REPOSITORY_SPECIFIC";
const CONFIGURATION_KEY_FILE_ROLLING: &str = "FILE_ROLLING";
const CONFIGURATION_KEY_DATE_FORMAT: &str = "DATE_FORMAT";
const CONFIGURATION_KEY_NOTE_TEMPLATE_FILE: &str = "NOTE_TEMPLATE_FILE";

/// A configuration is represented here.
#[derive(Debug, Clone)]
pub struct Configuration {
    /// Default notes directory. Affected by use_repository_specific.
    pub note_directory: String,

    /// Use also git specific notes.
    pub use_repository_specific: bool,

    /// File Rolling
    pub file_rolling: FileRolling,

    /// Path to the note template
    pub template_file: String,

    /// The note template
    pub note_template: NoteTemplate,
}

impl Configuration {
    /// returns the default note directory which is one of the following:
    /// 1. documents folder
    /// 2. user home folder
    /// 3. current folder!
    pub fn intial_note_directory() -> String {
        let initial_dir = if let Some(note_directory) = dirs::document_dir() {
            str!(note_directory.to_str().unwrap())
        } else if let Some(note_directory) = dirs::home_dir() {
            str!(note_directory.to_str().unwrap())
        } else {
            str!(env::current_dir().unwrap(), PathBuf)
        };
        log::debug!("Initial note directory: {}", initial_dir);
        initial_dir
    }

    /// configuration file path
    pub fn file_path() -> PathBuf {
        let path = Configuration::folder().join(CONFIGURATION_FILE_NAME);
        log::debug!("Configuration at {}", str!(path, PathBuf));
        path
    }

    /// configuration folder path
    pub fn folder() -> PathBuf {
        let dir = dirs::config_dir().unwrap().join(CONFIGURATION_FOLDER);
        log::debug!("Configuration folder at {}", str!(dir, PathBuf));
        dir
    }
}

impl Configuration {
    /// initializes the config.
    pub fn new() -> Self {
        let config_file = Configuration::file_path();
        if !config_file.exists() {
            // if no config exists, then create a new one with the defaults.
            let config = Configuration::default();
            let configuration_dir_not_created = format!(
                "Could not create configuration directory: {}",
                str!(Configuration::folder(), PathBuf)
            );
            create_dir_all(Configuration::folder()).expect(&configuration_dir_not_created);

            Configuration::save(&config);
            log::info!("Configuration with default values created");
            config
        } else {
            // read config if it already exists
            let configuration_not_open = format!(
                "Could not open configuration file: {}",
                str!(&config_file, PathBuf)
            );
            let c_file = File::open(&config_file).expect(&configuration_not_open);
            let c_buffer = BufReader::new(c_file);
            let c_raw_lines: Vec<String> = BufRead::lines(c_buffer).map(|l| l.unwrap()).collect();
            let mut config = Configuration::from(c_raw_lines);

            let template = match File::open(&config.template_file) {
                Ok(t_file) => {
                    let t_buffer = BufReader::new(t_file);
                    let t_raw_lines: Vec<String> =
                        BufRead::lines(t_buffer).map(|l| l.unwrap()).collect();
                    Some(t_raw_lines.join(LINE_ENDING))
                }
                Err(err) => {
                    log::error!(
                        "Could not read template at {}: {:?}",
                        &config.template_file,
                        err
                    );
                    None
                }
            };
            if let Some(note_template) = template {
                config.note_template.template = note_template;
            }
            log::info!("Read existing configuration");
            config
        }
    }

    pub fn save(configuration: &Configuration) {
        let configuration_not_created = format!(
            "Could not create configuration file: {}",
            &configuration.template_file
        );
        let t_file = File::create(&configuration.template_file).expect(&configuration_not_created);
        let t_buffer = &mut BufWriter::new(t_file);
        Write::write_all(t_buffer, configuration.note_template.template.as_ref())
            .expect("Could not write note template.");

        let config_file = Configuration::file_path();

        let c_file = File::create(&config_file).expect("Could not create configuration file");
        let c_buffer = &mut BufWriter::new(c_file);
        Write::write_all(c_buffer, configuration.to_string().as_ref())
            .expect("Could not write configuration");
    }
}

/// Defaults for Configuration
impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            note_directory: Configuration::intial_note_directory(),
            use_repository_specific: false,
            file_rolling: FileRolling::Daily,
            template_file: str!(NoteTemplate::initial_file_path(), PathBuf),
            note_template: NoteTemplate::default(),
        }
    }
}

/// ToString for Configuration File write.
impl ToString for Configuration {
    fn to_string(&self) -> String {
        str!(format!(
            indoc! {
            "{0}={1}
            {2}={3}
            {4}={5}
            {6}={7}
            {8}={9}
            "},
            CONFIGURATION_KEY_NOTE_DIRECTORY,
            &self.note_directory,
            CONFIGURATION_KEY_USE_REPOSITORY_SPECIFIC,
            &self.use_repository_specific,
            CONFIGURATION_KEY_FILE_ROLLING,
            &self.file_rolling,
            CONFIGURATION_KEY_DATE_FORMAT,
            &self.note_template.date_format,
            CONFIGURATION_KEY_NOTE_TEMPLATE_FILE,
            &self.template_file
        ))
    }
}

/// Parse configuration from read lines as vec<string>.
impl From<Vec<String>> for Configuration {
    fn from(lines: Vec<String>) -> Self {
        let mut note_directory = str!("");
        let mut date_format = str!("");
        let mut template_file = str!("");
        let mut use_repository_specific = false;
        let mut file_rolling = FileRolling::Daily;
        for line in lines {
            let config: Vec<&str> = line.split('=').collect();
            let value = str::replace(config[1], '"', "");
            if config[0].starts_with(CONFIGURATION_KEY_NOTE_DIRECTORY) {
                note_directory = value;
            } else if config[0].starts_with(CONFIGURATION_KEY_USE_REPOSITORY_SPECIFIC) {
                if let Ok(val) = value.parse() {
                    use_repository_specific = val;
                }
            } else if config[0].starts_with(CONFIGURATION_KEY_FILE_ROLLING) {
                if let Ok(val) = value.parse() {
                    file_rolling = val;
                }
            } else if config[0].starts_with(CONFIGURATION_KEY_DATE_FORMAT) {
                date_format = value;
            } else if config[0].starts_with(CONFIGURATION_KEY_NOTE_TEMPLATE_FILE) {
                template_file = value;
            }
        }
        Self {
            note_directory,
            use_repository_specific,
            file_rolling,
            template_file,
            note_template: NoteTemplate {
                date_format,
                ..NoteTemplate::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::{configuration::Configuration, file_rolling::FileRolling};

    #[test]
    fn when_configuration_default_called_then_default_configuration_is_returned() {
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
        assert!(!configuration.use_repository_specific);
    }
}
