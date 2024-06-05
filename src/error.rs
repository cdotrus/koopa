use std::path::PathBuf;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("destination {0:?} already exists")]
    DestinationExists(PathBuf),
    #[error("missing '=' sign")]
    VariableParseError,
    #[error("destination base path {0:?} is does not exist")]
    DestinationMissingDirectories(PathBuf),
}
