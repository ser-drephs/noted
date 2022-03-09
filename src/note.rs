use std::fmt::{self, Display, Formatter};

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

#[cfg(test)]
mod tests {
    use crate::note::Note;

    macro_rules! test_note {
        ($name:ident, $($a:expr, $e:expr),+) => {
            #[test]
            fn $name() {
                $({
                    let note = Note::from($a);
                    assert_eq!($e, note);
                })*
            }
        };
    }

    test_note!(
        when_created_from_empty_vec_then_default_note_is_returned,
        Vec::new(),
        Note::default()
    );

    test_note!(
        when_created_from_one_argument_then_note_without_tags_is_returned,
        ["Sample note"].to_vec(),
        Note {
            content: "Sample note".to_string(),
            ..Default::default()
        }
    );

    test_note!(
        when_created_from_two_arguments_then_note_with_tag_is_returned,
        ["Sample note", "tag1"].to_vec(),
        Note {
            content: "Sample note".to_string(),
            tags: vec!["tag1".to_string()]
        }
    );

    test_note!(
        when_created_from_multiple_arguments_then_note_with_multiple_tags_is_returned,
        ["Sample note", "tag1", "tag2", "tag3", "tag4"].to_vec(),
        Note {
            content: "Sample note".to_string(),
            tags: vec![
                "tag1".to_string(),
                "tag2".to_string(),
                "tag3".to_string(),
                "tag4".to_string()
            ]
        }
    );
}
