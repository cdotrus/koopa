use crate::{
    shell::{Key, Shell, Value},
    Error,
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

pub const CONFIG_DIR: &str = ".koopa";
pub const IGNORE_FILE: &str = ".koopaignore";
pub const CONFIG_FILE: &str = "shells.toml";

use ignore::gitignore::Gitignore;

#[derive(Debug)]
pub struct IgnoreFile {
    inner: Option<Gitignore>,
}

impl IgnoreFile {
    pub fn new() -> Self {
        Self { inner: None }
    }

    pub fn load(p: &PathBuf) -> Result<Self, Error> {
        let ignore_file = p.join(IGNORE_FILE);
        if ignore_file.exists() == true && ignore_file.is_file() == true {
            let _ = match std::fs::read_to_string(&ignore_file) {
                Ok(r) => r,
                Err(e) => return Err(Error::FileRead(ignore_file, Error::lowerize(e.to_string()))),
            };
            let (globs, err) = Gitignore::new(&ignore_file);
            if let Some(e) = err {
                return Err(Error::GitIgnoreParse(
                    p.to_path_buf(),
                    Error::lowerize(e.to_string()),
                ));
            }
            Ok(Self { inner: Some(globs) })
        } else {
            Ok(Self { inner: None })
        }
    }

    /// Checks if the given filepath is included. If there is no public list,
    /// then it will always return true.
    pub fn is_ignored(&self, path: &Path) -> bool {
        match &self.inner {
            Some(ig) => ig
                .matched_path_or_any_parents(path, path.is_dir())
                .is_ignore(),
            None => false,
        }
    }

    pub fn exists(&self) -> bool {
        self.inner.is_some()
    }
}

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

#[derive(Debug)]
pub struct Config {
    root: PathBuf,
    data: ConfigFile,
    ignore: IgnoreFile,
}

impl Config {
    pub fn new(p: PathBuf) -> Result<Self, Error> {
        let root = p.join(CONFIG_DIR);
        Ok(Self {
            data: ConfigFile::load(&root)?,
            ignore: IgnoreFile::load(&root)?,
            root: root,
        })
    }

    /// Attempts to look inside this configuration to see if a relative path
    /// exists and produces the new resolved path if so.
    pub fn resolve_source(&self, p: &PathBuf) -> Option<PathBuf> {
        if p.is_relative() == true {
            let potential_path = self.root.join(p);
            if potential_path.exists() == true {
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

    pub fn get_sources(&self) -> Vec<(PathBuf, PathBuf)> {
        let mut entries = Vec::new();
        let _ = Self::visit_dirs(&self.root, &mut entries, true, &self.ignore);
        entries.sort();
        // compile into pairs with relative path and full path
        entries
            .into_iter()
            .map(|f| (f.strip_prefix(&self.root).unwrap().to_path_buf(), f))
            .collect()
    }

    pub fn visit_dirs(
        dir: &Path,
        cb: &mut Vec<PathBuf>,
        skip_hidden: bool,
        ignore: &IgnoreFile,
    ) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                // ignore hidden files if true
                if ignore.is_ignored(&path) == false {
                    if skip_hidden == false
                        || entry.file_name().to_string_lossy().starts_with('.') == false
                    {
                        if path.is_dir() {
                            // allow this directory to be a source
                            cb.push(entry.path());
                            Self::visit_dirs(&path, cb, skip_hidden, ignore)?;
                        } else {
                            if skip_hidden == false || entry.file_name() != CONFIG_FILE {
                                // allow this file to be a source
                                cb.push(entry.path());
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
