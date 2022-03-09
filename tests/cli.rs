use indoc::indoc;
use noted::{cli::Cli, command::Command, str};

macro_rules! test_command {
    ($name:ident, $($s:expr, $o:expr),+) => {
        #[test]
        fn $name() {
            $({
                let res = Cli::parse($s.iter()).unwrap();
                assert_eq!(
                $o,
                res.command
            )})*
        }
    };
}

#[test]
fn cli_no_args_prints_help() {
    let err = Cli::parse(["noted"].iter()).unwrap_err();
    let expected = indoc!(
        "Take notes using CLI

            USAGE:
                noted [FLAGS] <note> [tag]...
                noted <SUBCOMMAND>

            FLAGS:
                -o               Open note file in default editor after writing
                -d               Set the level of verbosity
                -h, --help       Prints help information
                -v, --version    Prints version information

            ARGS:
                <note>      Note to take
                <tag>...    Tags for note

            SUBCOMMANDS:
                create    Create note file and open in default editor
                open      Opens note file in default editor
                search    Search for a specific string in notes
                config    Open configuration in default editor
                help      Prints this message or the help of the given subcommand(s)"
    );
    assert!(
        err.message.contains(expected),
        "Expected: {:?}, Got: {:?}",
        expected,
        err.message
    );
}

#[test]
fn cli_arg_help() {
    let err = Cli::parse(["noted", "help"].iter()).unwrap_err();
    let expected = indoc!(
        "Take notes using CLI

            USAGE:
                noted [FLAGS] <note> [tag]...
                noted <SUBCOMMAND>

            FLAGS:
                -o               Open note file in default editor after writing
                -d               Set the level of verbosity
                -h, --help       Prints help information
                -v, --version    Prints version information

            ARGS:
                <note>      Note to take
                <tag>...    Tags for note

            SUBCOMMANDS:
                create    Create note file and open in default editor
                open      Opens note file in default editor
                search    Search for a specific string in notes
                config    Open configuration in default editor
                help      Prints this message or the help of the given subcommand(s)"
    );
    assert!(err.message.contains(expected));
}

#[test]
fn cli_arg_help_create() {
    let err = Cli::parse(["noted", "help", "create"].iter()).unwrap_err();
    let expected = indoc!(
        "Creates a new note file in the configured note directory and opens it in default editor.

            USAGE:
                noted create [FLAGS] <filename>

            FLAGS:
                -d            Set the level of verbosity
                -h, --help    Prints help information

            ARGS:
                <filename>    File name for created note file"
    );
    assert!(err.message.contains(expected));
}

#[test]
fn cli_arg_help_open() {
    let err = Cli::parse(["noted", "help", "open"].iter()).unwrap_err();
    let expected = indoc!(
            "Open the current note file in the default editor.

            Depending on the configuration the current note file may also be repository specific.
            If filename is provided, a note file matching the pattern will be searched in the configured note directory.

            USAGE:
                noted open [FLAGS] [filename]

            FLAGS:
                -d            Set the level of verbosity
                -h, --help    Prints help information

            ARGS:
                <filename>    Provide filename for note to open"
        );
    assert!(err.message.contains(expected));
}

#[test]
fn cli_arg_help_search() {
    let err = Cli::parse(["noted", "help", "search"].iter()).unwrap_err();
    let expected = indoc!(
        "Search for a specific string in notes using the provided RegEx pattern.

            USAGE:
                noted search [FLAGS] <pattern> [file filter]

            FLAGS:
                -d            Set the level of verbosity
                -h, --help    Prints help information
                -t, --tag     Search only for tags

            ARGS:
                <pattern>        Search pattern
                <file filter>    File filter pattern"
    );
    assert!(err.message.contains(expected));
}

#[test]
fn cli_arg_help_config() {
    let err = Cli::parse(["noted", "help", "config"].iter()).unwrap_err();
    let expected = indoc!(
        "Open configuration in default editor

            USAGE:
                noted config [FLAGS]

            FLAGS:
                -d            Set the level of verbosity
                -h, --help    Prints help information"
    );
    assert!(err.message.contains(expected));
}

#[test]
fn cli_arg_help_short_flag() {
    let err = Cli::parse(["noted", "create", "-h"].iter()).unwrap_err();
    assert!(err.message.contains(
        "Creates a new note file in the configured note directory and opens it in default editor."
    ));
    assert!(err.message.contains("File name for created note file"));
}

#[test]
fn cli_arg_help_long_flag() {
    let err = Cli::parse(["noted", "create", "--help"].iter()).unwrap_err();
    assert!(err.message.contains(
        "Creates a new note file in the configured note directory and opens it in default editor."
    ));
    assert!(err.message.contains(
        "A note file with the provided name is created in the configured note directory."
    ));
}

#[test]
fn cli_verbosity_default() {
    let res = Cli::parse(["noted", "take some note"].iter()).unwrap();
    assert_eq!(log::LevelFilter::Warn, res.verbosity);
}

#[test]
fn cli_verbosity_info() {
    let res = Cli::parse(["noted", "-d", "take some note"].iter()).unwrap();
    assert_eq!(log::LevelFilter::Info, res.verbosity);
}

#[test]
fn cli_verbosity_debug() {
    let res = Cli::parse(["noted", "-dd", "take some note"].iter()).unwrap();
    assert_eq!(log::LevelFilter::Debug, res.verbosity);
}

#[test]
fn cli_verbosity_trace() {
    let res = Cli::parse(["noted", "-ddd", "file"].iter()).unwrap();
    assert_eq!(log::LevelFilter::Trace, res.verbosity);
}

#[test]
fn cli_verbosity_subcommand_default() {
    let res = Cli::parse(["noted", "create", "file"].iter()).unwrap();
    assert_eq!(log::LevelFilter::Warn, res.verbosity);
}

#[test]
fn cli_verbosity_subcommand_info() {
    let res = Cli::parse(["noted", "create", "-d", "file"].iter()).unwrap();
    assert_eq!(log::LevelFilter::Info, res.verbosity);
}

#[test]
fn cli_verbosity_subcommand_debug() {
    let res = Cli::parse(["noted", "create", "-dd", "file"].iter()).unwrap();
    assert_eq!(log::LevelFilter::Debug, res.verbosity);
}

#[test]
fn scli_verbosity_ubcommand_trace() {
    let res = Cli::parse(["noted", "create", "-ddd", "file"].iter()).unwrap();
    assert_eq!(log::LevelFilter::Trace, res.verbosity);
}

test_command!(
    cli_take_note,
    ["noted", "take some note"],
    Command::Note {
        open_after_write: false,
        note: str!("take some note"),
        tags: Vec::new()
    }
);

test_command!(
    cli_take_note_open_short,
    ["noted", "take some note", "-o"],
    Command::Note {
        open_after_write: true,
        note: str!("take some note"),
        tags: Vec::new()
    }
);

test_command!(
    cli_take_note_with_single_tag,
    ["noted", "take some note", "Tag1"],
    Command::Note {
        open_after_write: false,
        note: str!("take some note"),
        tags: [str!("Tag1")].to_vec()
    }
);

test_command!(
    cli_take_note_with_multiple_tags,
    ["noted", "take some note", "Tag1", "Tag2"],
    Command::Note {
        open_after_write: false,
        note: str!("take some note"),
        tags: [str!("Tag1"), str!("Tag2")].to_vec()
    }
);

test_command!(
    cli_take_note_with_multiple_tags_open,
    ["noted", "take some note", "Tag1", "Tag2", "-o"],
    Command::Note {
        open_after_write: true,
        note: str!("take some note"),
        tags: [str!("Tag1"), str!("Tag2")].to_vec()
    }
);

test_command!(
    cli_take_note_with_multiple_tags_open_mixed,
    ["noted", "take some note", "Tag1", "-o", "Tag2", "Tag3"],
    Command::Note {
        open_after_write: true,
        note: str!("take some note"),
        tags: [str!("Tag1"), str!("Tag2"), str!("Tag3")].to_vec()
    }
);

#[test]
fn cli_create_no_args_prints_help_about_required_arguments() {
    let err = Cli::parse(["noted", "create"].iter()).unwrap_err();
    assert!(err
        .message
        .contains("The following required arguments were not provided"));
    assert!(err.message.contains("filename"));
}

test_command!(
    cli_create_with_filename,
    ["noted", "create", "test"],
    Command::Create {
        filename: str!("test")
    }
);

test_command!(
    cli_create_alias_c,
    ["noted", "c", "test"],
    Command::Create {
        filename: str!("test")
    }
);

test_command!(
    cli_create_alias_new,
    ["noted", "new", "test"],
    Command::Create {
        filename: str!("test")
    }
);

test_command!(
    cli_create_alias_n,
    ["noted", "n", "test"],
    Command::Create {
        filename: str!("test")
    }
);

test_command!(
    cli_open_without_filename,
    ["noted", "open"],
    Command::Open { filename: None }
);

test_command!(
    cli_open_with_filename,
    ["noted", "open", "file"],
    Command::Open {
        filename: Some(str!("file"))
    }
);

test_command!(
    cli_open_alias_o,
    ["noted", "open"],
    Command::Open { filename: None }
);

test_command!(
    cli_open_alias_edit,
    ["noted", "edit"],
    Command::Open { filename: None }
);

test_command!(alias_e, ["noted", "e"], Command::Open { filename: None });

test_command!(
    cli_open_alias_view,
    ["noted", "view"],
    Command::Open { filename: None }
);

#[test]
fn cli_search_no_args_prints_help_about_required_arguments() {
    let err = Cli::parse(["noted", "search"].iter()).unwrap_err();
    assert!(err
        .message
        .contains("The following required arguments were not provided"));
    assert!(err.message.contains("pattern"));
}

test_command!(
    cli_search_pattern,
    ["noted", "search", "xyz*"],
    Command::Search {
        tag: false,
        pattern: str!("xyz*"),
        file_pattern: None,
        output_to_file: false
    }
);

test_command!(
    cli_search_pattern_tag,
    ["noted", "search", "--tag", "xyz*"],
    Command::Search {
        tag: true,
        pattern: str!("xyz*"),
        file_pattern: None,
        output_to_file: false
    }
);

test_command!(
    cli_search_pattern_tag_filepattern,
    ["noted", "search", "--tag", "xyz*", "*samplefile*"],
    Command::Search {
        tag: true,
        pattern: str!("xyz*"),
        file_pattern: Some(str!("*samplefile*")),
        output_to_file: false
    }
);

test_command!(
    cli_search_pattern_tag_short,
    ["noted", "search", "-t", "xyz*"],
    Command::Search {
        tag: true,
        pattern: str!("xyz*"),
        file_pattern: None,
        output_to_file: false
    }
);

test_command!(
    cli_search_alias_s,
    ["noted", "s", "-t", "xyz*"],
    Command::Search {
        tag: true,
        pattern: str!("xyz*"),
        file_pattern: None,
        output_to_file: false
    }
);

test_command!(
    cli_search_alias_grep,
    ["noted", "grep", "xyz*"],
    Command::Search {
        tag: false,
        pattern: str!("xyz*"),
        file_pattern: None,
        output_to_file: false
    }
);

test_command!(
    cli_search_alias_find,
    ["noted", "find", "xyz*"],
    Command::Search {
        tag: false,
        pattern: str!("xyz*"),
        file_pattern: None,
        output_to_file: false
    }
);

test_command!(
    cli_search_alias_f,
    ["noted", "f", "xyz*"],
    Command::Search {
        tag: false,
        pattern: str!("xyz*"),
        file_pattern: None,
        output_to_file: false
    }
);

test_command!(cli_config, ["noted", "config"], Command::Config);

// Known Limitaion: Currently it is not possible (at least I didn't find anythin suitable) to disable InferSubcommands in clap.
// Therefore this bug will persist.
//     use {Cli, Command};

//     test_command!(
//         command_create_like_defaults_to_note,
//         ["noted", "creat undefined"],
//         Command::Note {
//             open_after_write: false,
//             note: "creat undefined".to_string(),
//             tags: Vec::new()
//         }
//     );

//     test_command!(
//         command_open_like_defaults_to_note,
//         ["noted", "ope undefined"],
//         Command::Note {
//             open_after_write: false,
//             note: "ope undefined".to_string(),
//             tags: Vec::new()
//         }
//     );

//     test_command!(
//         command_search_like_defaults_to_note,
//         ["noted", "searc undefined"],
//         Command::Note {
//             open_after_write: false,
//             note: "searc undefined".to_string(),
//             tags: Vec::new()
//         }
//     );

//     test_command!(
//         command_config_like_defaults_to_note,
//         ["noted", "conf undefined"],
//         Command::Note {
//             open_after_write: false,
//             note: "conf undefined".to_string(),
//             tags: Vec::new()
//         }
//     );
