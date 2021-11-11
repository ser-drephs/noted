# Noted <!-- omit in toc -->

Reimplementation of [scottashipp's "noted"](https://github.com/scottashipp/noted) using Rust.

- [Features](#features)
- [Getting Started](#getting-started)
- [Usage](#usage)
    - [Other Commands](#other-commands)
        - [Option "-o"](#option--o)
        - [New](#new)
        - [Open](#open)
        - [Find](#find)
        - [Config](#config)
        - [Version](#version)
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

#### Option "-o"

Append the option `-o` to the noted command to open the notes file after writing the note.

Example:
```pwsh
noted "my note" -o
```
Writes the note `my note` and opens the note file in your default editor.

#### New

Creates a new note file with the provided file name in the configured [`NOTE_DIRECTORY`](#note_directory) and opens it in your default editor:
```pwsh
noted new <note file name>

noted n <note file name>
```

Example:
```pwsh
noted new noted-todos
```
Will create a file called `noted-todos.md` and open it in your default editor.

Alternative:
```pwsh
noted create <note file name>
```

**Limitation:** Ignores `USE_REPOSITORY_SPECIFIC` setting!

#### Open

Open notes in your default editor:
```pwsh
noted open

noted o
```

Open specific notes file in your default editor<sup>*</sup>:
```pwsh
noted open <filename>

noted o <filename>
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

Alternatives:
```pwsh
noted view

noted edit
```

<sup>* The first file that matches the filename will be opened.</sup>

<sup>** Supports wildcards.</sup>

#### Find

Todo: Not supprted yet!

Find a note containing the provided text:
```pwsh
noted find [<options>] <pattern> [<file filter>]

noted f [<options>] <pattern> [<file filter>]
```

Example:
```pwsh
noted find "*later*"

noted f "*later*"
```

Options:
- `tag`: search for tags

Alternatives:
```pwsh
noted grep [<options>] <pattern> [<file filter>]

noted search [<options>] <pattern> [<file filter>]
```

todo: second argument "-t"/"tag" or "-d"/"date"

#### Config

Open configuration file:

```pwsh
noted config
```

More information about configuration in section [Configuration](#configuration).

#### Version

Show the current version:

```pwsh
noted version
```

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
