use crate::{
    shell::{Key, Shell, Value},
    Error,
};
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

pub const CONFIG_DIR: &str = ".koopa";
pub const CONFIG_FILE: &str = "shells.toml";

#[derive(Debug, PartialEq, Deserialize)]
#[serde(transparent, deny_unknown_fields)]
pub struct ConfigFile {
    shells: HashMap<Key, Value>,
}

impl ConfigFile {
    pub fn new() -> Self {
        Self {
            shells: HashMap::new(),
        }
    }

    fn load(p: &PathBuf) -> Result<ConfigFile, Error> {
        let shell_file = p.join(CONFIG_FILE);
        if shell_file.exists() == true && shell_file.is_file() == true {
            let data = match std::fs::read_to_string(&shell_file) {
                Ok(r) => r,
                Err(e) => return Err(Error::FileRead(shell_file, Error::lowerize(e.to_string()))),
            };
            match toml::de::from_str(&data) {
                Ok(r) => Ok(r),
                Err(e) => Err(Error::TomlParse(shell_file, Error::lowerize(e.to_string()))),
            }
        } else {
            Ok(Self::new())
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Config {
    root: PathBuf,
    data: ConfigFile,
}

impl Config {
    pub fn new(p: PathBuf) -> Result<Self, Error> {
        let root = p.join(CONFIG_DIR);
        Ok(Self {
            data: ConfigFile::load(&root)?,
            root: root,
        })
    }

    /// Attempts to look inside this configuration to see if a relative path
    /// exists and produces the new resolved path if so.
    pub fn resolve_source(&self, p: &PathBuf) -> Option<PathBuf> {
        if p.is_relative() == true {
            let potential_path = self.root.join(p);
            if potential_path.exists() == true && potential_path.is_file() == true {
                Some(potential_path)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_shells(&self) -> Vec<Shell> {
        self.data
            .shells
            .clone()
            .into_iter()
            .map(|(k, v)| Shell::from((k.into_koopa_key(), v)))
            .collect()
    }
}
