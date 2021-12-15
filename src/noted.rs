mod integration_tests;
mod tests;

use indoc::indoc;

#[cfg(windows)]
pub const LINE_ENDING: &str = "\r\n";
#[cfg(not(windows))]
pub const LINE_ENDING: &str = "\n";

const CONFIGURATION_FOLDER: &str = "noted";
const CONFIGURATION_FILE_NAME: &str = "noted.config";

const CONFIGURATION_KEY_NOTE_DIRECTORY: &str = "NOTE_DIRECTORY";
const CONFIGURATION_KEY_USE_REPOSITORY_SPECIFIC: &str = "USE_REPOSITORY_SPECIFIC";
const CONFIGURATION_KEY_FILE_ROLLING: &str = "FILE_ROLLING";
const CONFIGURATION_KEY_DATE_FORMAT: &str = "DATE_FORMAT";
const CONFIGURATION_KEY_NOTE_TEMPLATE_FILE: &str = "NOTE_TEMPLATE_FILE";

const NOTES_FILE_NAME: &str = "notes.md";

#[macro_export]
macro_rules! vec_of_strings {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

#[macro_export]
macro_rules! str {
    ($x:expr, PathBuf) => {
        $x.to_str().unwrap().to_string()
    };
    ($x:expr) => {
        $x.to_string()
    };
}

trait Initialize {
    fn new() -> Self;
}

/// File Rolling to determinate the note file rolling cycle.
#[derive(Debug, PartialEq, Clone)]
pub enum FileRolling {
    /// new file every day
    Daily,
    /// new file every week
    Week,
    /// new file every month
    Month,
    /// new file every year
    Year,
    /// never create a new file
    Never,
}

impl std::str::FromStr for FileRolling {
    type Err = std::io::Error;
    fn from_str(input: &str) -> std::result::Result<FileRolling, Self::Err> {
        let i: &str = &input.to_lowercase();
        log::debug!("Try parsing '{}' to file rolling", &i);
        match i {
            "daily" => Ok(FileRolling::Daily),
            "week" => Ok(FileRolling::Week),
            "month" => Ok(FileRolling::Month),
            "year" => Ok(FileRolling::Year),
            "never" => Ok(FileRolling::Never),
            _ => Err({
                let fmt = format!("unable to parse file rolling from '{}'", input);
                std::io::Error::new(std::io::ErrorKind::InvalidInput, fmt)
            }),
        }
    }
}

#[cfg(not(tarpaulin_include))]
impl std::fmt::Display for FileRolling {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// The Note markdown template is represented here.
#[derive(Debug, Clone)]
pub struct NoteTemplate {
    /// The note template.
    ///
    /// ## Example
    ///
    /// ```md
    /// ---
    ///
    /// %date_format%
    ///
    /// %note%
    ///
    /// ---
    /// ```
    template: String,

    /// The date format. Supports [chrono::format::strftime](https://docs.rs/chrono/0.4.0/chrono/format/strftime/index.html).
    date_format: String,
}

impl NoteTemplate {
    fn initial_file_path() -> std::path::PathBuf {
        Configuration::folder().join("notedtemplate.md")
    }
}

impl Default for NoteTemplate {
    fn default() -> Self {
        Self {
            template: str!(indoc! {
                "%date_format%

            %note%

            %tags%"
            }),
            date_format: str!("%F %T"),
        }
    }
}

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
    fn intial_note_directory() -> String {
        let initial_dir = if let Some(note_directory) = dirs::document_dir() {
            str!(note_directory.to_str().unwrap())
        } else if let Some(note_directory) = dirs::home_dir() {
            str!(note_directory.to_str().unwrap())
        } else {
            str!(std::env::current_dir().unwrap(), PathBuf)
        };
        log::debug!("Initial note directory: {}", initial_dir);
        initial_dir
    }

    /// configuration file path
    fn file_path() -> std::path::PathBuf {
        let path = Configuration::folder().join(CONFIGURATION_FILE_NAME);
        log::debug!("Configuration at {}", str!(path, PathBuf));
        path
    }

    /// configuration folder path
    fn folder() -> std::path::PathBuf {
        let dir = dirs::config_dir().unwrap().join(CONFIGURATION_FOLDER);
        log::debug!("Configuration folder at {}", str!(dir, PathBuf));
        dir
    }
}

impl Initialize for Configuration {
    /// initializes the config.
    fn new() -> Self {
        let config_file = Configuration::file_path();
        if !config_file.exists() {
            // if no config exists, then create a new one with the defaults.
            let config = Configuration::default();
            let configuration_dir_not_created = format!(
                "Could not create configuration directory: {}",
                str!(Configuration::folder(), PathBuf)
            );
            std::fs::create_dir_all(Configuration::folder()).expect(&configuration_dir_not_created);
            let configuration_not_created = format!(
                "Could not create configuration file: {}",
                &config.template_file
            );
            let t_file =
                std::fs::File::create(&config.template_file).expect(&configuration_not_created);
            let t_buffer = &mut std::io::BufWriter::new(t_file);
            std::io::Write::write_all(t_buffer, config.note_template.template.as_ref())
                .expect("Could not write note template.");

            let c_file =
                std::fs::File::create(&config_file).expect("Could not create configuration file");
            let c_buffer = &mut std::io::BufWriter::new(c_file);
            std::io::Write::write_all(c_buffer, config.to_string().as_ref())
                .expect("Could not write configuration");
            log::info!("Configuration with default values created");
            config
        } else {
            // read config if it already exists
            let configuration_not_open = format!(
                "Could not open configuration file: {}",
                str!(&config_file, PathBuf)
            );
            let c_file = std::fs::File::open(&config_file).expect(&configuration_not_open);
            let c_buffer = std::io::BufReader::new(c_file);
            let c_raw_lines: Vec<String> = std::io::BufRead::lines(c_buffer)
                .map(|l| l.unwrap())
                .collect();
            let mut config = Configuration::from(c_raw_lines);

            let template = match std::fs::File::open(&config.template_file) {
                Ok(t_file) => {
                    let t_buffer = std::io::BufReader::new(t_file);
                    let t_raw_lines: Vec<String> = std::io::BufRead::lines(t_buffer)
                        .map(|l| l.unwrap())
                        .collect();
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

/// Represents the markdown file
struct Markdown {
    notefile: NoteFile,
}

impl From<std::path::PathBuf> for Markdown {
    fn from(path: std::path::PathBuf) -> Self {
        Markdown {
            notefile: NoteFile::from(path),
        }
    }
}

impl From<&std::path::PathBuf> for Markdown {
    fn from(path: &std::path::PathBuf) -> Self {
        Markdown {
            notefile: NoteFile::from(path.clone()),
        }
    }
}

impl Markdown {
    /// Write the formated note to the markdown file
    pub fn write(
        self,
        note: &FormatedNote,
    ) -> std::result::Result<std::path::PathBuf, std::io::Error> {
        match std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .truncate(false)
            .open(&self.notefile.file)
        {
            Ok(mut file) => match std::io::Write::write_all(&mut file, note.content.as_bytes()) {
                Ok(_) => {
                    log::debug!("Appending to note file: {}", &self.notefile.file);
                    Ok(std::path::PathBuf::from(self.notefile.file))
                }
                Err(err) => {
                    log::error!(
                        "Could not write to note file at {}: {:?}",
                        &self.notefile.file,
                        &err
                    );
                    Err(err)
                }
            },
            Err(err) => {
                log::error!(
                    "Could not create or append to note file at {}: {:?}",
                    &self.notefile.file,
                    &err
                );
                Err(err)
            }
        }
    }

    /// Reads notes from the markdown file.
    fn search(
        arguments: SearchArguments,
        configuration: &Configuration,
    ) -> std::result::Result<Vec<(String, u64, String)>, std::io::Error> {
        log::info!("Search string: '{}'", arguments.regex);
        if arguments.regex.is_empty() {
            let err_msg = "Search string is empty";
            log::error!("{}", err_msg);

            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                err_msg,
            ))
        } else {
            // first find all files that match the file pattern!
            let files = NoteFile::target_by_pattern(
                if let Some(file_regex) = &arguments.file_regex {
                    log::info!("Limit files to: '{}'", file_regex);
                    file_regex
                } else {
                    "*"
                },
                &std::path::PathBuf::from(&configuration.note_directory),
            );
            // if no files where found or there is an error, don't continue
            if files.is_err() {
                Err(files.err().unwrap())
            } else {
                let search_string = if arguments.tags_only {
                    format!("#{}", arguments.regex)
                } else {
                    // Format limiter as defined in SearchResult::to_table
                    let outer_limits = (45 - arguments.regex.len()) / 2;
                    format!(".?{{0,{0}}}{1}.?{{0,{0}}}", outer_limits, arguments.regex)
                };
                log::debug!("Using the following RegEx: {:?}", &search_string);
                let matcher = grep::regex::RegexMatcher::new(&search_string).unwrap();
                let mut matches: Vec<(String, u64, String)> = vec![];
                for file in files.unwrap() {
                    // in the found files vec search for the provided pattern.
                    let mut buffer = String::new();
                    let mut f = std::fs::File::open(&file).unwrap();
                    std::io::Read::read_to_string(&mut f, &mut buffer).unwrap();
                    grep::searcher::Searcher::new()
                        .search_slice(
                            &matcher,
                            buffer.as_bytes(),
                            grep::searcher::sinks::UTF8(|lnum, line| {
                                let mymatch =
                                    grep::matcher::Matcher::find(&matcher, line.as_bytes())?
                                        .unwrap();
                                // append each found occurence to the matches vec with filename, line and occurence.
                                matches.push((
                                    str!(file, PathBuf),
                                    lnum,
                                    line[mymatch].to_string(),
                                ));
                                Ok(true)
                            }),
                        )
                        .unwrap();
                }
                log::debug!("Found {} occurences", &matches.len());
                Ok(matches)
            }
        }
    }
}

/// Represents the formated note
struct FormatedNote {
    content: String,
}

/// A note is represented here.
#[derive(Debug, Clone, PartialEq)]
struct Note {
    /// Content of the note
    pub content: String,
    /// Tags of the note. Can be empty.
    pub tags: Vec<String>,
}

impl From<&str> for Note {
    fn from(content: &str) -> Self {
        Note {
            content: content.to_owned(),
            ..Default::default()
        }
    }
}

impl From<Vec<&str>> for Note {
    fn from(content: Vec<&str>) -> Self {
        if content.is_empty() {
            Note::default()
        } else {
            Note {
                content: content[0].to_owned(),
                tags: content[1..].iter().map(|s| s.to_string()).collect(),
            }
        }
    }
}

impl Default for Note {
    fn default() -> Self {
        Self {
            content: "".to_string(),
            tags: Vec::new(),
        }
    }
}

#[cfg(not(tarpaulin_include))]
impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Note {
    /// formats the note based on the provided format and returns a formated note to be written into a file.
    pub fn format(self, format: &NoteTemplate) -> FormatedNote {
        let now = chrono::Local::now();
        let mut note = format.template.to_string();
        note = note.replace(
            "%date_format%",
            &now.format(&format.date_format.to_string()).to_string(),
        );
        note = note.replace("%note%", &self.content);
        if note.contains("%tags%") {
            let tags = if !&self.tags.is_empty() {
                format!("#{}", &self.tags.join(";#"))
            } else {
                "".to_string()
            };
            note = note.replace("%tags%", &tags);
        }
        let formated_note = format!("{1}{0}{0}---{0}", LINE_ENDING, &note.trim());
        log::debug!("Writing note: {:?}", &formated_note);
        FormatedNote {
            content: formated_note,
        }
    }
}

/// Represents search arguments
#[derive(Debug, PartialEq)]
struct SearchArguments {
    regex: String,
    tags_only: bool,
    file_regex: Option<String>,
}

impl Default for SearchArguments {
    fn default() -> Self {
        SearchArguments {
            regex: str!(""),
            file_regex: None,
            tags_only: false,
        }
    }
}

/// Represents search results
#[derive(Debug)]
struct SearchResult {}

impl SearchResult {
    fn to_table(matches: Vec<(String, u64, String)>) -> Vec<String> {
        log::debug!("Formating {} occurences to a table", &matches.len());
        let mut table = vec![SearchResult::fmt("File", "Line", "Content")];
        for lines in matches {
            // cut filename to 30 chars. replace beginning if needed.
            let filepath = if lines.0.len() >= 30 {
                format!("...{}", &lines.0[&lines.0.len() - 27..])
            } else {
                lines.0
            };
            // cut occurence to 45 chars. replace end if needed
            let content = if lines.2.len() >= 45 {
                format!("{}...", &lines.2[0..42])
            } else {
                lines.2
            };
            table.push(SearchResult::fmt(&filepath, &str!(lines.1), &content));
        }
        for x in &table {
            println!("{}", x);
        }
        table
    }

    /// Write search results to file insted of stdout.
    /// Only used internally for tests!
    #[cfg(not(tarpaulin_include))]
    fn write(result: &[String]) -> std::result::Result<std::path::PathBuf, std::io::Error> {
        let log = std::path::PathBuf::from("search_result.txt");
        let file_write_error = format!("Could not write file at {}", str!(log, PathBuf));
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .truncate(false)
            .open(&log)
            .expect(&file_write_error);
        match std::io::Write::write_all(&mut file, result.join(LINE_ENDING).as_bytes()) {
            Ok(_) => Ok(log),
            Err(err) => Err(err),
        }
    }

    /// formatting of the search results table
    fn fmt(first: &str, second: &str, third: &str) -> String {
        format!("{0: <30} | {1: <4} | {2: <45}", first, second, third)
            .trim_end()
            .to_string()
    }
}

/// represents a note file
#[derive(Debug, PartialEq)]
struct NoteFile {
    file: String,
}

impl From<&FileRolling> for NoteFile {
    fn from(val: &FileRolling) -> Self {
        use FileRolling::*;
        let now = chrono::Local::now();
        log::debug!("Note file name based on file rolling: {}", val);
        NoteFile {
            file: match val {
                Daily => format!("{}.md", &now.format("%Y-%m-%d")),
                Week => format!("{}.md", &now.format("%Y-%W")),
                Month => format!("{}.md", &now.format("%Y-%m")),
                Year => format!("{}.md", &now.format("%Y")),
                Never => NOTES_FILE_NAME.to_string(),
            },
        }
    }
}

impl From<std::path::PathBuf> for NoteFile {
    fn from(path: std::path::PathBuf) -> Self {
        NoteFile {
            file: str!(path, PathBuf),
        }
    }
}

impl NoteFile {
    /// search for note files in provided base_dir for pattern by regex
    fn target_by_pattern(
        regex: &str,
        base_dir: &std::path::Path,
    ) -> std::result::Result<Vec<std::path::PathBuf>, std::io::Error> {
        if !regex.is_empty() {
            let options = glob::MatchOptions {
                case_sensitive: false,
                ..Default::default()
            };

            log::debug!("Search for '{}' in {}.", regex, str!(base_dir, PathBuf));
            match glob::glob_with(regex, options) {
                Ok(paths) => {
                    let mut filter_result: Vec<_> = paths
                        .filter_map(Result::ok)
                        .map(|f| base_dir.join(f))
                        .collect();
                    filter_result.sort_unstable();

                    log::debug!(
                        "Found {} files that match pattern '{}' in {}.",
                        filter_result.len(),
                        regex,
                        str!(base_dir, PathBuf)
                    );

                    if filter_result.is_empty() {
                        let err_msg = "Result is empty.";
                        log::error!("{}", err_msg);
                        Err(std::io::Error::new(std::io::ErrorKind::NotFound, err_msg))
                    } else {
                        Ok(filter_result)
                    }
                }
                Err(err) => Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
            }
        } else {
            let err_msg = "Pattern is empty";
            log::error!("{}", err_msg);
            Err(std::io::Error::new(std::io::ErrorKind::Other, err_msg))
        }
    }

    fn first_target_by_pattern(
        regex: &str,
        base_dir: &std::path::Path,
    ) -> std::result::Result<std::path::PathBuf, std::io::Error> {
        match NoteFile::target_by_pattern(regex, base_dir){
            Ok(files) => {
                Ok(files.get(0).unwrap().clone())
            },
            Err(err) => Err(err),
        }
    }

    /// get current target note file based on configuration
    fn target(configuration: &Configuration) -> std::path::PathBuf {
        let try_repo_specific = if configuration.use_repository_specific {
            let cur_dir = std::env::current_dir().unwrap();
            log::debug!(
                "Configured repository specific note file. Searching for note file in: {}",
                str!(cur_dir, PathBuf)
            );
            match &git2::Repository::discover(cur_dir) {
                Ok(repo) => {
                    log::debug!("Repository found. Using repository specific note file.");
                    Some(path_clean::PathClean::clean(
                        &repo.path().join("..").join(NOTES_FILE_NAME),
                    ))
                }
                Err(_) => {
                    log::info!("Repository not found.");
                    None
                }
            }
        } else {
            None
        };
        if let Some(repo_specific) = try_repo_specific {
            repo_specific
        } else {
            let notefile = NoteFile::from(&configuration.file_rolling);
            std::path::PathBuf::from(&configuration.note_directory).join(notefile.file)
        }
    }

    /// create a custom target file
    fn custom_target(filename: &str, configuration: &Configuration) -> std::path::PathBuf {
        if filename.is_empty() {
            NoteFile::target(configuration)
        } else {
            let new_filename = if !filename.ends_with(".md") {
                format!("{}.md", filename)
            } else {
                filename.to_string()
            };
            std::path::PathBuf::from(&configuration.note_directory).join(new_filename)
        }
    }
}

/// represents the command to be executed
#[derive(Debug, PartialEq)]
pub enum Command {
    Default,
    Note {
        open_after_write: bool,
        note: String,
        tags: Vec<String>,
    },
    Create {
        filename: String,
    },
    Open {
        filename: Option<String>,
    },
    Search {
        tag: bool,
        pattern: String,
        file_pattern: Option<String>,
        /// Only used for tests!
        output_to_file: bool,
    },
    Config,
}

#[cfg(not(tarpaulin_include))]
impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Command {
    pub fn invoke(
        &self,
        configuration: Option<Configuration>,
    ) -> std::result::Result<Option<std::path::PathBuf>, std::io::Error> {
        use Command::*;

        // create or read configuration
        let configuration = if let Some(config) = configuration {
            config
        } else {
            Configuration::new()
        };

        log::debug!("Invoke command: {}", self);

        match self {
            Default => todo!(),
            Note {
                open_after_write,
                note,
                tags,
            } => {
                // create note entry
                let note = crate::noted::Note {
                    content: note.to_string(),
                    tags: tags.to_vec(),
                }
                .format(&configuration.note_template);
                // write to markdown
                match Markdown::from(NoteFile::target(&configuration)).write(&note) {
                    Ok(filepath) => Ok(if open_after_write.to_owned() {
                        Some(filepath)
                    } else {
                        None
                    }),
                    Err(err) => Err(err),
                }
            }
            Create { filename } => {
                // create note entry
                let note = crate::noted::Note::default().format(&configuration.note_template);
                match Markdown::from(&NoteFile::custom_target(filename, &configuration))
                    .write(&note)
                {
                    Ok(file) => Ok(Some(file)),
                    Err(err) => Err(err),
                }
            }
            Open { filename } => {
                // if additional string is provided in arguments try to find a file with the filename in note directory
                let file = if let Some(pattern) = filename {
                    NoteFile::first_target_by_pattern(
                        pattern,
                        &std::path::PathBuf::from(&configuration.note_directory),
                    )
                    .unwrap()
                } else {
                    // othwise open current file
                    NoteFile::target(&configuration)
                };
                Ok(Some(file))
            }
            Search {
                tag,
                pattern,
                file_pattern,
                output_to_file,
            } => {
                match Markdown::search(
                    SearchArguments {
                        regex: pattern.to_owned(),
                        tags_only: tag.to_owned(),
                        file_regex: file_pattern.to_owned(),
                    },
                    &configuration,
                ) {
                    Ok(res) => {
                        let result = SearchResult::to_table(res);

                        if output_to_file.to_owned() {
                            SearchResult::write(&result).unwrap();
                        }
                        Ok(None)
                    }
                    Err(err) => Err(err),
                }
            }
            Config => {
                // open config in default editor
                Ok(Some(Configuration::file_path()))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Cli {
    pub command: Command,
    pub verbosity: log::LevelFilter,
}

impl Default for Cli {
    fn default() -> Self {
        Cli {
            command: Command::Default,
            verbosity: log::LevelFilter::Warn,
        }
    }
}

#[cfg(not(tarpaulin_include))]
impl std::fmt::Display for Cli {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> Cli {
    const NOTE: &'a str = "note";
    const TAG: &'a str = "tag";
    const OPT_VERBOSE: &'a str = "d";

    const CMD_ARG_FILENAME: &'a str = "filename";

    const CMD_CREATE: &'a str = "create";

    const CMD_OPEN: &'a str = "open";

    const CMD_SEARCH: &'a str = "search";
    const CMD_SEARCH_ARG_TAG: &'a str = "tag";
    const CMD_SEARCH_ARG_PATTERN: &'a str = "pattern";
    const CMD_SEARCH_ARG_FILEPATTERH: &'a str = "file filter";

    const CMD_CONFIG: &'a str = "config";

    const DEFAULT_SUBCMD_APPSETTINGS: &'a [clap::AppSettings] =
        &[clap::AppSettings::DisableVersion];

    #[cfg(not(tarpaulin_include))]
    fn initialize() -> clap::App<'a, 'a> {
        clap::App::new("Noted CLI")
            .version(clap::crate_version!())
            .settings(&[
                clap::AppSettings::VersionlessSubcommands,
                clap::AppSettings::ArgsNegateSubcommands,
                clap::AppSettings::ArgRequiredElseHelp,
                clap::AppSettings::ColorAuto,
                clap::AppSettings::DeriveDisplayOrder,
                clap::AppSettings::GlobalVersion,
                clap::AppSettings::SubcommandsNegateReqs
            ])
            .about("Take notes using CLI")
            .author("ser-drephs <ser-drephs@gmail.com>")
            .version_short("v")
            .args(&[
                clap::Arg::with_name(Cli::NOTE)
                    .help("Note to take")
                    .required(true)
                    .index(1),
                clap::Arg::with_name(Cli::TAG)
                    .help("Tags for note")
                    .required(false)
                    .multiple(true)
                    .index(2),
                clap::Arg::with_name(Cli::CMD_OPEN)
                    .short("o")
                    .takes_value(false)
                    .help("Open note file in default editor after writing"),
                clap::Arg::with_name(Cli::OPT_VERBOSE)
                    .short(Cli::OPT_VERBOSE)
                    .multiple(true)
                    .takes_value(false)
                    .help("Set the level of verbosity")
                    .global(true),
            ])
            .subcommands([
                clap::SubCommand::with_name(Cli::CMD_CREATE)
                    .about("Create note file and open in default editor")
                    .long_about("Creates a new note file in the configured note directory and opens it in default editor.")
                    .settings(Cli::DEFAULT_SUBCMD_APPSETTINGS)
                    .aliases(&["c", "new", "n"])
                    .arg(
                        clap::Arg::with_name(Cli::CMD_ARG_FILENAME)
                            .help("File name for created note file")
                            .long_help("A note file with the provided name is created in the configured note directory.")
                            .required(true)
                            .index(1)
                        ),
                clap::SubCommand::with_name(Cli::CMD_OPEN)
                    .about("Opens note file in default editor")
                    .long_about(indoc!("Open the current note file in the default editor.

                    Depending on the configuration the current note file may also be repository specific.
                    If filename is provided, a note file matching the pattern will be searched in the configured note directory."))
                    .settings(Cli::DEFAULT_SUBCMD_APPSETTINGS)
                    .aliases(&["o", "edit", "e", "view"])
                    .arg(
                        clap::Arg::with_name(Cli::CMD_ARG_FILENAME)
                            .help("Provide filename for note to open")
                            .long_help("Given this argument a note file with the provided name is searched in the configured note directory and opened in your default editor.")
                            .required(false)
                            .index(1),
                    ),
                clap::SubCommand::with_name(Cli::CMD_SEARCH)
                    .about("Search for a specific string in notes")
                    .long_about("Search for a specific string in notes using the provided RegEx pattern.")
                    .settings(Cli::DEFAULT_SUBCMD_APPSETTINGS)
                    .aliases(&["s", "grep", "find", "f"])
                    .args(&[
                        clap::Arg::with_name(Cli::CMD_SEARCH_ARG_TAG)
                            .short("t")
                            .long("tag")
                            .help("Search only for tags")
                            .required(false)
                            .takes_value(false),
                        clap::Arg::with_name(Cli::CMD_SEARCH_ARG_PATTERN)
                            .help("Search pattern")
                            .required(true)
                            .index(1),
                        clap::Arg::with_name(Cli::CMD_SEARCH_ARG_FILEPATTERH)
                            .help("File filter pattern")
                            .required(false)
                            .index(2)
                    ]),
                clap::SubCommand::with_name(Cli::CMD_CONFIG)
                    .about("Open configuration in default editor")
                    .settings(Cli::DEFAULT_SUBCMD_APPSETTINGS)
                    .alias("c"),
            ])
    }

    pub fn parse<I, T>(args: I) -> Result<Self, clap::Error>
    where
        I: Iterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let mut command: Cli = Default::default();

        let app = Cli::initialize();

        let matches = app.get_matches_from_safe(args)?;

        command.verbosity = match &matches.occurrences_of(Cli::OPT_VERBOSE) {
            0 => log::LevelFilter::Warn,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        };

        command.command = match matches.subcommand() {
            (&_, None) => Command::Note {
                open_after_write: matches.is_present(Cli::CMD_OPEN),
                note: matches.value_of(Cli::NOTE).unwrap().to_string(),
                tags: matches
                    .values_of(Cli::TAG)
                    .unwrap_or_default()
                    .map(|t| t.to_string())
                    .collect(),
            },
            (Cli::CMD_CREATE, Some(create)) => Command::Create {
                filename: create.value_of(Cli::CMD_ARG_FILENAME).unwrap().to_string(),
            },
            (Cli::CMD_OPEN, Some(open)) => Command::Open {
                filename: if open.is_present(Cli::CMD_ARG_FILENAME) {
                    Some(open.value_of(Cli::CMD_ARG_FILENAME).unwrap().to_string())
                } else {
                    None
                },
            },
            (Cli::CMD_SEARCH, Some(search)) => Command::Search {
                tag: search.is_present(Cli::CMD_SEARCH_ARG_TAG),
                pattern: search
                    .value_of(Cli::CMD_SEARCH_ARG_PATTERN)
                    .unwrap()
                    .to_string(),
                file_pattern: search
                    .value_of(Cli::CMD_SEARCH_ARG_FILEPATTERH)
                    .map(|file_pattern| str!(file_pattern)),
                output_to_file: false,
            },
            (Cli::CMD_CONFIG, Some(_)) => Command::Config,
            (&_, Some(&_)) => Command::Default,
        };

        Ok(command)
    }
}
