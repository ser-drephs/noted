mod tests;

use indoc::indoc;
use std::{io::Read, str::FromStr};

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

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

#[derive(Debug, PartialEq)] //, Clone)]
pub enum FileRolling {
    Daily,
    Week,
    Month,
    Year,
    Never,
}

impl std::str::FromStr for FileRolling {
    type Err = std::io::Error;
    fn from_str(input: &str) -> std::result::Result<FileRolling, Self::Err> {
        let i: &str = &input.to_lowercase();
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

impl std::fmt::Display for FileRolling {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, PartialEq)] //, Clone)]
pub enum Command {
    Direct,
    Create,
    Config,
    Open,
    Grep,
    Version,
    Help,
}

impl std::str::FromStr for Command {
    type Err = ();

    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        let i: &str = &input.to_lowercase();
        match i {
            "create" | "new" | "n" => Ok(Command::Create),
            "config" => Ok(Command::Config),
            "edit" | "view" | "open" | "o" => Ok(Command::Open),
            "grep" | "search" | "find" | "f" => Ok(Command::Grep),
            "version" | "v" => Ok(Command::Version),
            "?" | "h" | "help" => Ok(Command::Help),
            _ => Ok(Command::Direct),
        }
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PostCommand {
    Open,
    None,
}

impl std::str::FromStr for PostCommand {
    type Err = ();

    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        let i: &str = &input.to_lowercase();
        match i {
            "-o" => Ok(PostCommand::Open),
            _ => Ok(PostCommand::None),
        }
    }
}

impl std::fmt::Display for PostCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for PostCommand {
    fn default() -> Self {
        PostCommand::None
    }
}

/// A note is represented here.
#[derive(Debug, Default)]
pub struct Note {
    /// Content of the note
    pub content: String,
    /// Optional Tags of the note
    pub tags: Option<Vec<String>>,
    /// Optional Template for to_string
    pub template: NoteTemplate,
    /// Optional open note after creation
    pub post_create_command: PostCommand,
}

// impl From<Vec<String>> for Note {
//     fn from(arguments: Vec<String>) -> Self {
//         match &arguments.len().cmp(&1) {
//             std::cmp::Ordering::Equal => Note {
//                 content: str!(arguments[0]),
//                 ..Note::default()
//             },
//             std::cmp::Ordering::Greater => Note {
//                 content: str!(arguments[0]),
//                 tags: Some(arguments[1..].to_vec()),
//                 post_create_command: arguments
//                     .last()
//                     .map(|last_argument| PostCommand::from_str(last_argument).unwrap()),
//                 ..Note::default()
//             },
//             std::cmp::Ordering::Less => Note::default(),
//         }
//     }
// }

impl From<&Vec<String>> for Note {
    fn from(arguments: &Vec<String>) -> Self {
        match &arguments.len().cmp(&1) {
            std::cmp::Ordering::Equal => Note {
                content: str!(arguments[0]),
                ..Note::default()
            },
            std::cmp::Ordering::Greater => {
                let post_create_command = match arguments
                    .last()
                    .map(|last_argument| PostCommand::from_str(last_argument).unwrap())
                {
                    Some(res) => res,
                    None => PostCommand::None,
                };

                let tags = if post_create_command != PostCommand::None {
                    Some(arguments[1..arguments.len() - 1].to_vec())
                } else {
                    Some(arguments[1..].to_vec())
                };

                Note {
                    content: str!(arguments[0]),
                    tags,
                    post_create_command,
                    ..Note::default()
                }
            }
            std::cmp::Ordering::Less => Note::default(),
        }
    }
}

impl ToString for Note {
    fn to_string(&self) -> String {
        let now = chrono::Local::now();
        let mut note = self.template.template.to_string();
        note = note.replace(
            "%date_format%",
            &now.format(&self.template.date_format.to_string())
                .to_string(),
        );
        note = note.replace("%note%", &self.content);
        if note.contains("%tags%") {
            let tags: String = if let Some(opt_tags) = &self.tags {
                format!("#{}", opt_tags.join(";#"))
            } else {
                "".to_string()
            };
            note = note.replace("%tags%", &tags);
        }
        format!("{1}{0}{0}---{0}", LINE_ENDING, note.trim())
    }
}

// impl std::fmt::Display for Note {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(
//             f,
//             "Content: {0}, Tags: #{1}",
//             &self.content,
//             if let Some(tags) = &self.tags {
//                 tags.join(", #")
//             } else {
//                 str!("")
//             }
//         )
//     }
// }

/// The Note markdown template is represented here.
#[derive(Debug)]
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
#[derive(Debug)]
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
    fn intial_note_directory() -> String {
        if let Some(note_directory) = dirs::document_dir() {
            str!(note_directory.to_str().unwrap())
        } else if let Some(note_directory) = dirs::home_dir() {
            str!(note_directory.to_str().unwrap())
        } else {
            str!("")
        }
    }

    fn file_path() -> std::path::PathBuf {
        Configuration::folder().join(CONFIGURATION_FILE_NAME)
    }

    fn folder() -> std::path::PathBuf {
        dirs::config_dir().unwrap().join(CONFIGURATION_FOLDER)
    }
}

impl Initialize for Configuration {
    fn new() -> Self {
        let config_file = Configuration::file_path();
        if !config_file.exists() {
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
            config
        } else {
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
                    println!("Could not read template: {}", err);
                    None
                }
            };
            if let Some(note_template) = template {
                config.note_template.template = note_template;
            }
            config
        }
    }
}

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

struct MarkdownSearchArguments {
    regex: String,
    search_type: MarkdownSearchType,
    file_regex: Option<String>,
}

impl Default for MarkdownSearchArguments {
    fn default() -> Self {
        MarkdownSearchArguments {
            regex: str!(""),
            file_regex: None,
            search_type: MarkdownSearchType::Default,
        }
    }
}

impl From<Vec<String>> for MarkdownSearchArguments {
    fn from(arguments: Vec<String>) -> Self {
        if arguments.is_empty() {
            MarkdownSearchArguments::default()
        } else {
            let search_type = MarkdownSearchType::from_str(&arguments[0]).unwrap();
            let other_args = if search_type != MarkdownSearchType::Default {
                arguments[1..].to_vec()
            } else {
                arguments.to_vec()
            };
            match &other_args.len().cmp(&1) {
                std::cmp::Ordering::Equal => MarkdownSearchArguments {
                    regex: str!(other_args[0]),
                    search_type,
                    ..MarkdownSearchArguments::default()
                },
                std::cmp::Ordering::Greater => {
                    let file_regex = str!(other_args[1]);
                    MarkdownSearchArguments {
                        regex: str!(other_args[0]),
                        search_type,
                        file_regex: Some(file_regex),
                    }
                }
                std::cmp::Ordering::Less => MarkdownSearchArguments {
                    search_type,
                    ..MarkdownSearchArguments::default()
                },
            }
        }
    }
}

#[derive(Debug, PartialEq)] //, Clone)]
pub enum MarkdownSearchType {
    Tags,
    Default,
}

impl std::str::FromStr for MarkdownSearchType {
    type Err = ();

    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        let i: &str = &input.to_lowercase();
        match i {
            "tag" | "tags" | "t" => Ok(MarkdownSearchType::Tags),
            _ => Ok(MarkdownSearchType::Default),
        }
    }
}

impl std::fmt::Display for MarkdownSearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct MarkdownSearchResult {}

impl MarkdownSearchResult {
    fn to_table(matches: Vec<(String, u64, String)>) -> Vec<String> {
        let mut table = vec![MarkdownSearchResult::fmt("File", "Line", "Content")];
        for lines in matches {

            let filepath = if lines.0.len() >= 30 {
                format!("...{}", &lines.0[&lines.0.len()-27..])
            } else {
                lines.0
            };
            let content = if lines.2.len() >= 45 {
                format!("{}...", &lines.2[0..42])
            } else {
                lines.2
            };
            table.push(MarkdownSearchResult::fmt(
                &filepath,
                &str!(lines.1),
                &content,
            ));
        }
        for x in &table{
            println!("{}", x);
        }
        table
    }

    fn fmt(first: &str, second: &str, third: &str) -> String {
        format!("{0: <30} | {1: <4} | {2: <45}", first, second, third)
    }
}

struct Markdown {}

impl Markdown {
    fn write(
        note: &Note,
        file_path: &std::path::Path,
    ) -> std::result::Result<PostCommand, std::io::Error> {
        // let file = Markdown::target(configuration);

        let file_write_error = format!("Could not write file at {}", str!(file_path, PathBuf));
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .truncate(false)
            .open(&file_path)
            .expect(&file_write_error);
        let post_create_command = note.post_create_command.clone();
        match std::io::Write::write_all(&mut file, note.to_string().as_bytes()) {
            Ok(_) => Ok(post_create_command),
            Err(err) => Err(err),
        }
    }

    /// Reads notes from the markdown file.
    fn search(
        arguments: MarkdownSearchArguments,
        configuration: &Configuration,
    ) -> std::result::Result<Vec<(String, u64, String)>, std::io::Error> {
        println!("WARNING: NOT YET IMPLEMENTED");
        println!("Search string: '{}'", arguments.regex);
        if arguments.regex.is_empty() {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "search string is empty",
            ))
        } else {
            let files = Markdown::search_target(&arguments, configuration);
            if files.is_err() {
                Err(files.err().unwrap())
            } else {
                let search_string = match arguments.search_type {
                    MarkdownSearchType::Tags => format!("#{}", arguments.regex),
                    _ => arguments.regex,
                };
                let matcher = grep::regex::RegexMatcher::new(&search_string).unwrap();
                let mut matches: Vec<(String, u64, String)> = vec![];
                for file in files.unwrap() {
                    let mut buffer = String::new();
                    let mut f = std::fs::File::open(&file).unwrap();
                    f.read_to_string(&mut buffer).unwrap();
                    grep::searcher::Searcher::new()
                        .search_slice(
                            &matcher,
                            buffer.as_bytes(),
                            grep::searcher::sinks::UTF8(|lnum, line| {
                                let mymatch =
                                    grep::matcher::Matcher::find(&matcher, line.as_bytes())?
                                        .unwrap();
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
                // for found in matches{
                //     println!("Found: {1}{0}Line: {2}{0}", LINE_ENDING, &found.0, found.1);
                // }
                // println!("Search type: '{}'", arguments.search_type);
                // if let Some(filter) = arguments.file_regex {
                //     println!("Search file filter: {}", filter);
                // } else {
                //     println!("No file filter.");
                // }
                // println!("Search string: {}", arguments.join(" | "));
                // let file = File::open(&self.note_file).expect("File not found");

                // BufReader::new(file)
                //     .lines()
                //     .map(|l| l.expect("Cloud not read file"))
                //     .collect()
                // todo!("search for pattern in multiple files")
                Ok(matches)
            }
        }
    }

    fn target(configuration: &Configuration) -> std::path::PathBuf {
        let try_repo_specific = if configuration.use_repository_specific {
            match &git2::Repository::discover(std::env::current_dir().unwrap()) {
                Ok(repo) => Some(path_clean::PathClean::clean(
                    &repo.path().join("..").join(NOTES_FILE_NAME),
                )),
                Err(_) => None,
            }
        } else {
            None
        };
        if let Some(repo_specific) = try_repo_specific {
            repo_specific
        } else {
            let now = chrono::Local::now();
            let filename = match &configuration.file_rolling {
                // YEAR-MONTH-DAY.md
                FileRolling::Daily => {
                    format!("{}.md", &now.format("%Y-%m-%d"))
                }
                // YEAR-WEEKNUMBER.md
                FileRolling::Week => {
                    format!("{}.md", &now.format("%Y-%W"))
                }
                // YEAR-MONTH.md
                FileRolling::Month => {
                    format!("{}.md", &now.format("%Y-%m"))
                }
                // YEAR.md
                FileRolling::Year => {
                    format!("{}.md", &now.format("%Y"))
                }
                FileRolling::Never => NOTES_FILE_NAME.to_string(),
            };
            std::path::PathBuf::from(&configuration.note_directory).join(filename)
        }
    }

    fn taget_by_pattern(
        arguments: Vec<String>,
        configuration: &Configuration,
    ) -> Option<std::path::PathBuf> {
        // let note_file: Option<std::path::PathBuf> =
        let search_arguments = MarkdownSearchArguments {
            file_regex: if !arguments.is_empty() {
                Some(format!("{0}.md", arguments[0]))
            } else {
                None
            },
            ..Default::default()
        };
        match Markdown::search_target(&search_arguments, configuration) {
            Ok(res) => res.get(0).cloned(),
            Err(_) => None,
        }
    }

    fn custom_target(arguments: &[String], configuration: &Configuration) -> std::path::PathBuf {
        if arguments.is_empty() {
            Markdown::target(configuration)
        } else {
            let mut filename = arguments.get(0).unwrap().clone();
            if !filename.ends_with(".md") {
                filename = format!("{}.md", filename);
            }
            std::path::PathBuf::from(&configuration.note_directory).join(filename)
        }
    }

    fn search_target(
        arguments: &MarkdownSearchArguments,
        configuration: &Configuration,
    ) -> std::result::Result<Vec<std::path::PathBuf>, std::io::Error> {
        let options = glob::MatchOptions {
            case_sensitive: false,
            ..Default::default()
        };
        if let Some(file_regex) = &arguments.file_regex {
            let cur_dir = std::env::current_dir().unwrap();
            match glob::glob_with(file_regex, options) {
                Ok(paths) => {
                    let filter_result: Vec<_> = paths
                        .filter_map(Result::ok)
                        .map(|f| cur_dir.join(f))
                        .collect();
                    if filter_result.is_empty() {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!("file not found by pattern: '{}'", file_regex),
                        ))
                    } else {
                        Ok(filter_result)
                    }
                }
                Err(err) => Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
            }
        } else {
            Ok([Markdown::target(configuration)].to_vec())
        }
    }
}

pub struct CommandHandler {}

impl CommandHandler {

    pub fn execute(arguments: Vec<String>) -> std::io::Result<()>{
        let mut command = Command::Help;
        if arguments.len() > 1 {
            command = Command::from_str(&arguments[1]).unwrap();
        } else {
            // let command_arg = &args[1];
            println!("provide a note or a command");
        }

        CommandHandler::invoke(command, arguments[1..].to_vec())
    }

    fn invoke(
        command: Command,
        arguments: Vec<String>,
    ) -> std::result::Result<(), std::io::Error> {
        match command {
            Command::Direct => CommandHandler::write_note(arguments[0..].to_vec()),
            Command::Create => CommandHandler::create_note(arguments[1..].to_vec()),
            Command::Config => CommandHandler::open_config(),
            Command::Open => CommandHandler::open_notes(arguments[1..].to_vec()),
            Command::Grep => CommandHandler::search_notes(arguments[1..].to_vec()),
            Command::Version => CommandHandler::show_version(),
            Command::Help => CommandHandler::show_help(),
        }
    }

    fn create_note(arguments: Vec<String>) -> std::result::Result<(), std::io::Error> {
        // get configuration
        let configuration = Configuration::new();
        // create note entry
        let note = Note::from(&vec_of_strings!(""));
        // find target
        let file = Markdown::custom_target(&arguments, &configuration);
        // write to markdown
        match Markdown::write(&note, &file) {
            Ok(cmd) => CommandHandler::post_write_command(cmd, arguments),
            Err(err) => Err(err),
        }
    }

    fn write_note(arguments: Vec<String>) -> std::result::Result<(), std::io::Error> {
        // get configuration
        let configuration = Configuration::new();
        // create note entry
        let note = Note::from(&arguments);
        // find target
        let file = Markdown::target(&configuration);
        // write to markdown
        match Markdown::write(&note, &file) {
            Ok(_) => open::that(&file),
            Err(err) => Err(err),
        }
    }

    fn open_notes(arguments: Vec<String>) -> std::result::Result<(), std::io::Error> {
        // read configuration
        let configuration = Configuration::new();
        // if additional string is provided in  arguments
        if let Some(file) = Markdown::taget_by_pattern(arguments, &configuration) {
            open::that(file)
        } else {
            let custom_error =
                std::io::Error::new(std::io::ErrorKind::NotFound, "could not open note file");
            Err(custom_error)
        }
    }

    fn open_config() -> std::result::Result<(), std::io::Error> {
        let config_path = Configuration::file_path();
        if !config_path.exists() {
            Configuration::new();
        }
        // open config in default editor
        open::that(config_path)
    }

    fn search_notes(arguments: Vec<String>) -> std::result::Result<(), std::io::Error> {
        // get configuration
        let configuration = Configuration::new();
        // search for whatever
        let search_args = MarkdownSearchArguments::from(arguments);
        match Markdown::search(search_args, &configuration) {
            Ok(res) => {
                MarkdownSearchResult::to_table(res);
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn show_version() -> std::result::Result<(), std::io::Error> {
        println!("noted v{}", VERSION.unwrap_or("unknown"));
        Ok(())
    }

    fn show_help() -> std::result::Result<(), std::io::Error> {
        // todo: help verbessern
        let help_msg = indoc!("
        Usage:

        noted \"take a note\"
            - Takes a note with the content

        noted create / new \"filename\"
            - Creates a new note file with the provided filename

        noted edit / view
            - Opens todays notes

        noted grep / search [option] [filepattern]
            - Search though the notes [ Option switches: -t = Search Tags]

        noted config
            - Opens configuration file for edit

        noted version
            - Get current version
        ");
        println!("{}", help_msg);
        Ok(())
    }

    fn post_write_command(
        post_command: PostCommand,
        arguments: Vec<String>,
    ) -> std::result::Result<(), std::io::Error> {
        match post_command {
            PostCommand::Open => CommandHandler::open_notes(arguments),
            _ => Ok(()),
        }
    }
}
