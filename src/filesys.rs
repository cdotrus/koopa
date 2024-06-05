//! Project: Koopa
//! Module: filesys
//!
//! An abstraction layer between the application logic and low-level filesystem
//! functions.

use std::{path::PathBuf, str::FromStr};

#[derive(Debug, PartialEq)]
pub struct Path {
    raw: String,
    path: PathBuf,
}

impl FromStr for Path {
    type Err = <PathBuf as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            path: PathBuf::from_str(s)?,
            raw: s.to_string(),
        })
    }
}

impl From<String> for Path {
    fn from(value: String) -> Self {
        Self {
            raw: value.clone(),
            path: PathBuf::from(value),
        }
    }
}

impl Path {
    pub fn new() -> Self {
        Self {
            raw: String::new(),
            path: PathBuf::from(String::new()),
        }
    }

    pub fn is_relative(&self) -> bool {
        self.path.is_relative()
    }
}
