use std::fmt::{Formatter, Display, self};

use crate::{note_template::NoteTemplate, FormatedNote, LINE_ENDING};

/// A note is represented here.
#[derive(Debug, Clone, PartialEq)]
pub struct Note {
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
impl Display for Note {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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