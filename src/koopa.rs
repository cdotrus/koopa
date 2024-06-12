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
                true => {
                    let _ = cli.get::<PathBuf>(Arg::positional("src"));
                    PathBuf::new()
                }
            },
            dest: match list | version {
                false => cli.require(Arg::positional("dest"))?,
                true => {
                    let _ = cli.get::<PathBuf>(Arg::positional("dest"));
                    PathBuf::new()
                }
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
            println!("Sources:");
            // print the source files from .koopa
            let key_order: Vec<&PathBuf> = {
                let mut arr: Vec<&PathBuf> = koopa_sources.keys().collect();
                arr.sort();
                arr
            };
            key_order.iter().for_each(|&k| {
                println!(
                    "({}) {} -> {:?}",
                    if koopa_sources.get(k).unwrap().is_file() {
                        "f"
                    } else {
                        "d"
                    },
                    k.display(),
                    koopa_sources.get(k).unwrap()
                )
            });
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
        self.run(shells)
    }
}

impl Koopa {
    fn run(&self, mut shells: ShellMap) -> Result<(), AnyError> {
        // ensure the data is allowed to be moved to the destination
        Self::has_permission(&self.dest, self.force)?;

        // perform the copy operation
        let bytes_copied = match self.src.is_file() {
            true => Self::copy_file(&self.src, &self.dest, &shells, self.force, self.verbose)?,
            false => Self::copy_dir(&self.src, &self.dest, &mut shells, self.force, self.verbose)?,
        };

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

    /// Performs the copy operation for a directory. If the function fails,
    /// no files will be available.
    fn copy_dir(
        src: &PathBuf,
        dest: &PathBuf,
        shells: &mut ShellMap,
        force: bool,
        verbose: bool,
    ) -> Result<usize, AnyError> {
        // get all the sources
        let mut src_files: Vec<PathBuf> = Vec::new();

        match Config::visit_dirs(&src.as_path(), &mut src_files, false) {
            Ok(_) => (),
            Err(e) => return Err(Box::new(e))?,
        }

        // take only the files with us
        let src_files: Vec<PathBuf> = src_files.into_iter().filter(|f| f.is_file()).collect();

        // create the list of file destinations
        let mut dest_files = Vec::new();
        src_files
            .iter()
            .for_each(|f| dest_files.push(dest.join(f.strip_prefix(src).unwrap())));

        let mut bytes_copied = 0;

        if force == true && dest.exists() == true {
            // remove everything within the existing destintation
            match std::fs::remove_dir_all(&dest) {
                Ok(_) => (),
                Err(e) => return Err(Box::new(e))?,
            }
        }

        // create base directory
        match std::fs::create_dir_all(&dest) {
            Ok(_) => (),
            Err(e) => {
                // remove all intermediate progress
                match std::fs::remove_dir_all(&dest) {
                    Ok(_) => return Err(Box::new(e))?,
                    Err(e) => return Err(Box::new(e))?,
                }
            }
        }

        for i in 0..src_files.len() {
            let src_file = src_files.get(i).unwrap();
            let dest_file = dest_files.get(i).unwrap();

            // create any missing directories for destination
            match std::fs::create_dir_all(&dest_file.parent().unwrap()) {
                Ok(_) => (),
                Err(e) => {
                    // remove all intermediate progress
                    match std::fs::remove_dir_all(&dest) {
                        Ok(_) => return Err(Box::new(e))?,
                        Err(e) => return Err(Box::new(e))?,
                    }
                }
            }
            // set koopa.name for each file
            shells.merge(ShellMap::from(&vec![Shell::with(
                format!("{}{}", shell::KEY_PREFIX, "name"),
                Self::find_filename(dest_file).unwrap(),
            )]));

            bytes_copied += match Self::copy_file(&src_file, &dest_file, &shells, force, verbose) {
                Ok(b) => b,
                Err(e) => {
                    // remove all intermediate progress
                    match std::fs::remove_dir_all(&dest) {
                        Ok(_) => return Err(e),
                        Err(e) => return Err(Box::new(e))?,
                    }
                }
            }
        }
        Ok(bytes_copied)
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
    fn copy_file(
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
        let mut last_linebreak: Option<isize> = None;
        while let Some((i, c)) = stream.next() {
            // state transitions
            if c == '\n' {
                line_no += 1;
                last_linebreak = Some(i as isize);
            }
            match state {
                State::Normal => {
                    result.push(c);
                    if c == '{' {
                        col_no = (i as isize - last_linebreak.unwrap_or(-1)) as usize;
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
                            // multi-line values should maintain the same indentation
                            Some(val) => {
                                let indentation = if col_no == 0 { 0 } else { col_no - 1 };
                                let mut lines = val.as_str().split('\n');
                                result.push_str(lines.next().unwrap());
                                while let Some(line) = lines.next() {
                                    result.push_str(&format!(
                                        "\n{}{}",
                                        (0..indentation).map(|_| " ").collect::<String>(),
                                        line
                                    ));
                                }
                            }
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

    #[test]
    fn ut_translate_text_multiline_value() {
        let text = "hello {{ koopa.multi }} and all!";
        let mut shells = ShellMap::new();
        shells.insert(Shell::with(
            String::from("koopa.multi"),
            String::from("earth\nvenus\nmars"),
        ));
        assert_eq!(
            Koopa::translate(text, &shells, true, false).unwrap(),
            "hello earth
      venus
      mars and all!"
        );

        let text = "hello {{ koopa.multi }} and all!";
        let mut shells = ShellMap::new();
        shells.insert(Shell::with(
            String::from("koopa.multi"),
            String::from("earth\nvenus\nmars\n\n"),
        ));
        assert_eq!(
            Koopa::translate(text, &shells, true, false).unwrap(),
            "hello earth
      venus
      mars
      
       and all!"
        );

        let text = "hello\n{{ koopa.multi }} and all!";
        let mut shells = ShellMap::new();
        shells.insert(Shell::with(
            String::from("koopa.multi"),
            String::from("earth\n venus\nmars\n"),
        ));
        assert_eq!(
            Koopa::translate(text, &shells, true, false).unwrap(),
            "hello
earth
 venus
mars
 and all!"
        );
    }
}
