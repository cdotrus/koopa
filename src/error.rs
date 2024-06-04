use std::path::PathBuf;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("destination {0:?} already exists")]
    DestinationExists(PathBuf),
}
