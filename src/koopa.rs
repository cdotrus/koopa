#![allow(dead_code)]

use super::error::Error;
use super::help;
use super::shell::{Shell, ShellMap};
use crate::config::Config;
use crate::shell::{self, Key};
use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Command, Help};
use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::path::PathBuf;

type AnyError = Box<dyn std::error::Error>;

#[derive(Debug, PartialEq)]
pub struct Koopa {
    src: PathBuf,
    dest: PathBuf,
    force: bool,
    verbose: bool,
    version: bool,
    list: bool,
    ignore_home: bool,
    ignore_work: bool,
    shells: Vec<Shell>,
}

impl Command for Koopa {
    fn interpret(cli: &mut Cli<Memory>) -> cli::Result<Self> {
        // logic for interface priority to user manual and version shortcuts

        cli.help(Help::with(help::SHORT_HELP))?;
        let verbose = cli.check(Arg::flag("verbose"))?;
        if verbose == true {
            cli.help(Help::with(help::LONG_HELP))?;
        }
        cli.raise_help()?;
        cli.lower_help();

        let version = cli.check(Arg::flag("version"))?;
        let list = cli.check(Arg::flag("list"))?;

        cli.help(Help::with(help::SHORT_HELP))?;
        Ok(Self {
            verbose: cli.check(Arg::flag("verbose"))?,
            version: cli.check(Arg::flag("version"))?,
            force: cli.check(Arg::flag("force"))?,
            list: cli.check(Arg::flag("list"))?,
            ignore_work: cli.check(Arg::flag("ignore-work"))?,
            ignore_home: cli.check(Arg::flag("ignore-home"))?,
            shells: cli
                .get_all(Arg::option("shell").switch('s').value("key=value"))?
                .unwrap_or_default(),
            src: match list | version {
                false => cli.require(Arg::positional("src"))?,
                true => PathBuf::new(),
            },
            dest: match list | version {
                false => cli.require(Arg::positional("dest"))?,
                true => PathBuf::new(),
            },
        })
    }

    fn execute(mut self) -> proc::Result {
        if self.version == true {
            println!("{}", help::VERSION);
            return Ok(());
        }

        let mut shells = ShellMap::new();

        // start with the standard shells (blue shells)
        if self.list == false {
            shells.merge(ShellMap::from(&vec![Shell::with(
                format!("{}{}", shell::KEY_PREFIX, "name"),
                Self::find_filename(&self.dest)?,
            )]));
        }

        let mut koopa_sources: HashMap<PathBuf, PathBuf> = HashMap::new();

        // load configurations and shells from files (red shells)
        {
            let mut resolved_src = self.src.clone();

            // home directory (if exists)
            if self.ignore_home == false {
                if let Some(home) = home::home_dir() {
                    let home_config = Config::new(home)?;
                    if let Some(name) = home_config.resolve_source(&self.src) {
                        resolved_src = name;
                    }
                    shells.merge(ShellMap::from(&home_config.get_shells()));
                    koopa_sources.extend(home_config.get_sources().into_iter());
                }
            }

            // current working directory and its parent directories
            if self.ignore_work == false {
                let mut work_dirs = vec![std::env::current_dir()?];
                while let Some(p) = work_dirs.last().unwrap().parent() {
                    work_dirs.push(p.to_path_buf());
                }
                work_dirs.reverse();

                for dir in work_dirs {
                    let work_config = Config::new(dir)?;
                    if let Some(name) = work_config.resolve_source(&self.src) {
                        resolved_src = name;
                    }
                    shells.merge(ShellMap::from(&work_config.get_shells()));
                    koopa_sources.extend(work_config.get_sources().into_iter());
                }
            }

            if self.list == false {
                if self.src != resolved_src {
                    help::info(
                        format!("resolved source path to {:?}", resolved_src),
                        self.verbose,
                    );
                }
                self.src = resolved_src;
            }
        }

        // load shells from command-line (green shells)
        shells.merge(ShellMap::from(&self.shells));

        if self.list == true {
            println!("Files:");
            // print the source files from .koopa
            let key_order: Vec<&PathBuf> = {
                let mut arr: Vec<&PathBuf> = koopa_sources.keys().collect();
                arr.sort();
                arr
            };
            key_order
                .iter()
                .for_each(|&k| println!("{} -> {:?}", k.display(), koopa_sources.get(k).unwrap()));
            println!();
            println!("Shells:");
            // print the shells
            let key_order: Vec<&Key> = {
                let mut arr: Vec<&Key> = shells.inner().keys().collect();
                arr.sort();
                arr
            };
            key_order
                .iter()
                .for_each(|&k| println!("{} -> \"{}\"", k, shells.get(k).unwrap()));
            println!();
            return Ok(());
        }

        // run the command
        self.run(&shells)
    }
}

impl Koopa {
    fn run(&self, shells: &ShellMap) -> Result<(), AnyError> {
        // ensure the data is allowed to be moved to the destination
        Self::has_permission(&self.dest, self.force)?;
        // perform the copy operation
        let bytes_copied = Self::copy(&self.src, &self.dest, shells, self.force, self.verbose)?;
        // provide information back to the user that the operation was a success
        help::info(
            format!(
                "successfully koopa'ed {} bytes to {:?}",
                bytes_copied, self.dest
            ),
            self.verbose,
        );
        Ok(())
    }

    /// Attempts to acquire the string of the file name, minus its extension.
    fn find_filename(p: &PathBuf) -> Result<String, Error> {
        if let Some(p) = p.file_name() {
            if let Some(p) = p.to_str() {
                return Ok(String::from(p.split('.').into_iter().next().unwrap()));
            }
        }
        Err(Error::DestinationMissingFileName(p.clone()))
    }

    /// Peforms the copy operation, moving bytes from `src` to `dest` while replacing
    /// any known variables with their corresponding values.
    fn copy(
        src: &PathBuf,
        dest: &PathBuf,
        shells: &ShellMap,
        force: bool,
        verbose: bool,
    ) -> Result<usize, AnyError> {
        // read contents from source
        let read_words = match std::fs::read_to_string(&src) {
            Ok(r) => r,
            Err(e) => return Err(Error::FileRead(src.clone(), Error::lowerize(e.to_string())))?,
        };
        // translate any variables within the text
        let write_words = match Self::translate(&read_words, shells, force, verbose) {
            Ok(r) => r,
            Err(e) => {
                return Err(Error::TranslationFailed(
                    src.clone(),
                    Error::lowerize(e.to_string()),
                ))?
            }
        };

        let working_path = Path::new(".");
        let base_path = dest.parent().unwrap_or(&working_path);

        // place the contents at the destination
        match std::fs::write(&dest, &write_words) {
            Ok(_) => (),
            Err(e) => match force {
                false => {
                    return Err(Box::new(Error::DestinationMissingDirectories(
                        base_path.to_path_buf(),
                    )))
                }
                true => {
                    if e.kind() == io::ErrorKind::NotFound {
                        std::fs::create_dir_all(base_path)?;
                        std::fs::write(&dest, &write_words)?;
                    } else {
                        return Err(Box::new(e));
                    }
                }
            },
        }
        Ok(write_words.len())
    }

    /// Verifies the data is allowed to be placed at the destination path.
    fn has_permission(path: &PathBuf, ignore: bool) -> Result<(), Error> {
        match ignore == false && path.exists() == true {
            true => Err(Error::DestinationExists(path.clone())),
            false => Ok(()),
        }
    }

    /// Translates the string contents `text` with variable replacement.
    fn translate(
        text: &str,
        shells: &ShellMap,
        force: bool,
        verbose: bool,
    ) -> Result<String, Error> {
        enum State {
            Normal,
            L1,
            Replace,
            R1,
        }

        let mut result = String::with_capacity(text.len());
        let mut key = Key::new();
        let mut state = State::Normal;

        let mut stream = text.char_indices();
        let mut line_no: usize = 1;
        let mut col_no: usize = 1;
        while let Some((i, c)) = stream.next() {
            // state transitions
            if c == '\n' {
                line_no += 1;
                col_no = 1;
            }
            match state {
                State::Normal => {
                    result.push(c);
                    if c == '{' {
                        col_no = i + 1;
                        state = State::L1
                    }
                }
                State::L1 => match c {
                    '{' => {
                        result.pop();
                        state = State::Replace;
                    }
                    _ => {
                        result.push(c);
                        state = State::Normal;
                    }
                },
                State::Replace => {
                    key.push(c);
                    if c == '}' {
                        state = State::R1
                    }
                }
                State::R1 => match c {
                    '}' => {
                        key.pop();
                        if key.is_koopa_key() == true {
                            // make sure this key being read is valid
                            if let Some(e) = key.validate() {
                                return Err(Error::KeyInvalid(
                                    key.clone(),
                                    line_no,
                                    col_no,
                                    Error::lowerize(e.to_string()),
                                ));
                            }
                        }
                        // replace the variable with its value
                        match shells.get(&key) {
                            Some(val) => result.push_str(val.as_str()),
                            None => {
                                // make sure we know this is a missing key if recognized
                                if key.is_koopa_key() == true {
                                    if force == false {
                                        return Err(Error::KeyUnknown(
                                            key.clone(),
                                            line_no,
                                            col_no,
                                        ));
                                    } else {
                                        help::warning(
                                            format!("skipping unknown key {}", key),
                                            verbose,
                                        );
                                    }
                                }
                                result.push_str(&key.to_string())
                            }
                        }
                        // clean up the contents stored in the variable
                        key.clear();
                        state = State::Normal;
                    }
                    _ => {
                        key.push(c);
                        state = State::Replace;
                    }
                },
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn ut_has_permission_err() {
        let path = PathBuf::from("README.md");
        assert_eq!(
            Err(Error::DestinationExists(path.clone())),
            Koopa::has_permission(&path, false)
        );
    }

    #[test]
    fn ut_has_permission_ok() {
        let path = PathBuf::from("some_unnamed_file.txt.txt");
        assert_eq!(Ok(()), Koopa::has_permission(&path, false));

        let path = PathBuf::from("README.md");
        assert_eq!(Ok(()), Koopa::has_permission(&path, true));
    }

    #[test]
    fn ut_translate_text_ok() {
        let text = "hello {{ koopa.foo }} and {{ koopa.bar }}!";
        let mut shells = ShellMap::new();
        shells.insert(Shell::with(
            String::from("koopa.foo"),
            String::from("world"),
        ));
        assert_eq!(
            Koopa::translate(text, &shells, true, false).unwrap(),
            "hello world and {{ koopa.bar }}!"
        );

        let text = "hello {{ koopa.foo }} and {{ koopa.bar }}!";
        let mut shells = ShellMap::new();
        shells.insert(Shell::with(String::from("koopa.bar"), String::from("moon")));
        shells.insert(Shell::with(
            String::from("koopa.foo"),
            String::from("world"),
        ));
        assert_eq!(
            Koopa::translate(text, &shells, true, false).unwrap(),
            "hello world and moon!"
        );
    }

    #[test]
    fn ut_translate_text_err() {
        let text = "hello {{ koopa.foo }}!";
        let shells = ShellMap::new();
        assert_eq!(
            Koopa::translate(text, &shells, false, false),
            Err(Error::KeyUnknown(Key::from_str("koopa.foo").unwrap(), 1, 7))
        );
    }
}
