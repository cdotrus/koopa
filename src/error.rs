use super::shell::Key;
use std::path::PathBuf;

type LastError = String;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("destination {0:?} already exists")]
    DestinationExists(PathBuf),
    #[error("destination {0:?} is missing a file name")]
    DestinationMissingFileName(PathBuf),
    #[error("destination base path {0:?} does not exist")]
    DestinationMissingDirectories(PathBuf),
    #[error("could not parse shell due to missing '=' character")]
    ShellParseMissingEq,
    #[error("failed to koopa file {0:?}: {1}")]
    TranslationFailed(PathBuf, LastError),
    #[error("unknown key \"{0}\" at line {1} col {2}")]
    KeyUnknown(Key, usize, usize),
    #[error("invalid key \"{0}\" at line {1} col {2}: {3}")]
    KeyInvalid(Key, usize, usize, LastError),
    #[error("key \"{0}\" contains whitespace between characters")]
    KeyContainsWhitespace(String),
    #[error("key \"{0}\" contains newline character")]
    KeyContainsNewline(String),
    #[error("key \"{0}\" contains too many '.' characters (expected 1)")]
    KeyContainsMoreDots(String),
    #[error("failed to read shell file {0:?}: {1}")]
    TomlParse(PathBuf, LastError),
    #[error("failed to read file {0:?}: {1}")]
    FileRead(PathBuf, LastError),
}

impl Error {
    // Presents the message `s` without the first letter being capitalized.
    pub fn lowerize(s: String) -> String {
        s.char_indices()
            .into_iter()
            .map(|(i, c)| if i == 0 { c.to_ascii_lowercase() } else { c })
            .collect()
    }
}
