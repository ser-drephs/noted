# Noted <!-- omit in toc -->

Reimplementation of [scottashipp's "noted"](https://github.com/scottashipp/noted) using Rust.

- [Features](#features)
- [Getting Started](#getting-started)
- [Usage](#usage)
    - [Other Commands](#other-commands)
        - [Options](#options)
        - [Option "-o"](#option--o)
        - [Option "-d"](#option--d)
        - [Create](#create)
        - [Open](#open)
        - [Find](#find)
        - [Config](#config)
- [Configuration](#configuration)
    - [NOTE_DIRECTORY](#note_directory)
    - [DATE_FORMAT](#date_format)
    - [USE_REPOSITORY_SPECIFIC](#use_repository_specific)
    - [FILE_ROLLING](#file_rolling)
    - [NOTE_TEMPLATE_FILE](#note_template_file)

## Features

- Create markdown file with notes
- Easy CLI usage
- Timestamp for notes
- Tags for notes

## Getting Started

Place `noted.exe` in directory which can be discovered using your `%PATH%` variable.

## Usage

Take a note with the content 'my note':

```pwsh
noted "my note"
```

Creates a note that looks like this:

```markdown
2021-11-11 11:07:49

my note


---
```

Take another note with the content 'second note':
```pwsh
noted "my note"
```

Appends the note to todays file:
```markdown
2021-11-11 11:07:49

my note

---

2021-11-11 11:08:49

second note

---
```

Take note with the content 'second note' and tags:
```pwsh
noted "my note with a tag" my-tag
```

Appends the note to todays file:
```markdown
2021-11-11 11:07:49

my note

---

2021-11-11 11:08:49

second note

---

2021-11-11 11:09:49

my note with a tag

#my-tag

---
```

### Other Commands

You can always view the help for noted using either the help subcommand or the `--help` flag.

```pwsh
noted help
```

#### Options

The options can be globally applied, if not stated otherwise.

#### Option "-o"

This option only applies to taking a note.

Append the option `-o` to the noted command to open the notes file after writing the note.

Example:
```pwsh
noted "my note" -o
```
Writes the note `my note` and opens the note file in your default editor.

#### Option "-d"

Set the logging level of the command. By default only warnings and errors are shown.

Example 1:
```pwsh
noted "my note" -d
```
Will write more information to output.

#### Create

Creates a new note file with the provided file name in the configured [`NOTE_DIRECTORY`](#note_directory) and opens it in your default editor:
```pwsh
noted create <filename>
```

Example:
```pwsh
noted create noted-todos
```
Will create a file called `noted-todos.md` and open it in your default editor.

Alias:
```pwsh
noted c <filename>
noted new <filename>
noted n <filename>
```

**Limitation:** Ignores `USE_REPOSITORY_SPECIFIC` setting!

#### Open

Open notes in your default editor:
```pwsh
noted open
```

Open specific notes file in your default editor<sup>*</sup>:
```pwsh
noted open <filename>
```

Example 1:
```pwsh
noted open
```
Will open current note file in your default editor.

Example 2:
```pwsh
noted open 2021-03-12
```
Will try to open the file `2021-03-12.md` in your default editor<sup>*,**</sup>.

Alias:
```pwsh
noted o
noted edit
noted e
noted view
```

<sup>* The first file that matches the filename will be opened.</sup>

<sup>** Supports wildcards.</sup>

#### Find

Find a note containing the provided text:
```pwsh
noted search [FLAGS] <pattern> [file filter]
```

Options:
- `tag`: search only for tags

Example 1:
```pwsh
noted search "*later*"
```

Searches for `*later*` in all notes inside the configured note directory.

Example 2:
```pwsh
noted search -t "bug"
```

Searches for the tag `bug` in all notes inside the configured note directory.

Example 2:
```pwsh
noted search -t "bug" 2021-01*
```

Searches for the tag `bug` in all notes that match the filter "2021-01*" inside the configured note directory.

Alias:
```pwsh
noted s [FLAGS] <pattern> [file filter]
noted grep [FLAGS] <pattern> [file filter]
noted find [FLAGS] <pattern> [file filter]
noted f [FLAGS] <pattern> [file filter]
```

#### Config

Open configuration file:

```pwsh
noted config
```

More information about configuration in section [Configuration](#configuration).

## Configuration

The configuration file can be found inside the Platform Specific config folder. See table below:

Platform | Value
-- | --
Linux | $HOME/.config/noted
macOS | $HOME/Library/Application Support/noted
Windows | %APPDATA%\noted

### NOTE_DIRECTORY

Default storage for notes. This can be one static path for ex. `$HOME/notes` or depending on the configuration [USE_REPOSITORY_SPECIFIC](#use_repository_specific] will be the repository folder.

### DATE_FORMAT

Dete format for the note timestamp. Supported date and time formats can be [looked up here](https://docs.rs/chrono/0.4.0/chrono/format/strftime/index.html).

### USE_REPOSITORY_SPECIFIC

If you enable this, the notes will be placed into the repository where noted was invoked.

**Example:**

`USE_REPOSITORY_SPECIFIC=true` is configured.

```powershell
/sources/my_repository $ noted "sample"
```

Will create a note file inside the repository `my_repository`.

```powershell
/home $ noted "sample"
```

Will create a note file inside the directory configured in `NOTE_DIRECTORY`.

### FILE_ROLLING

This setting changes the file rolling behaviour. Supported values are:
- `Daily` (Default) - Creates a new file for each day. The file format is fixed to `YEAR-MONTH-DAY.md`.
- `Week` - Creates a new file for every week. The file format is fixed to `YEAR-WEEKNUMBER.md`.
- `Month` - Creates a new file every month. The file format is fixed to `YEAR-MONTH.md`.
- `Year` - Creates a new file every year. The file format is fixed to `YEAR.md`.
- `Never` - All notes are placed in one single file. The file is named `notes.md`.

### NOTE_TEMPLATE_FILE

Path to the note template. By default the template file is located in the same directory as the noted config.

You can add a custom template and configure it here.

Supported keywords for the template:
- `%date_format%` - Timestamp of the note as configured in [DATE_FORMAT](#date_format).
- `%note%` - The note.
- `%tags%` - Tags for the note.

**Note:** At the end of each note the delimiter `---` is added. No need ot put it in the template.

**Default Template:**

```md
%date_format%

%note%

%tags%
```
