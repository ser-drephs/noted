use std::{fmt::{Display, Formatter, self}, ffi};

use crate::command::Command;
use indoc::indoc;

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
impl Display for Cli {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
        T: Into<ffi::OsString> + Clone,
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
