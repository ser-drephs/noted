use std::path::PathBuf;

use crate::{str, configuration::Configuration};
use indoc::indoc;

const TEMPLATE_FILE_NAME: &str = "noted.template";

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
    pub template: String,

    /// The date format. Supports [chrono::format::strftime](https://docs.rs/chrono/0.4.0/chrono/format/strftime/index.html).
    pub date_format: String,
}

impl NoteTemplate {
    pub fn initial_file_path() -> PathBuf {
        Configuration::folder().join(TEMPLATE_FILE_NAME)
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