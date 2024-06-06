use super::shell::Key;
use std::path::PathBuf;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("destination {0:?} already exists")]
    DestinationExists(PathBuf),
    #[error("could not parse shell due to missing '=' sign")]
    ShellParseMissingEq,
    #[error("destination base path {0:?} is does not exist")]
    DestinationMissingDirectories(PathBuf),
    #[error("unknown key {0:?} at line {1} col {2}")]
    UnknownKey(Key, usize, usize),
    #[error("failed to koopa file {0:?}: {1}")]
    TranslationFailed(PathBuf, String),
}
