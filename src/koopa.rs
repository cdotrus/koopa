#![allow(dead_code)]

use super::error::Error;
use super::help;
use super::var::{VarMap, Variable};
use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Command, Help};
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
    variables: Vec<Variable>,
}

impl Command for Koopa {
    fn interpret(cli: &mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(help::SHORT_HELP))?;
        let verbose = cli.check(Arg::flag("verbose"))?;
        if verbose == true {
            cli.help(Help::with(help::LONG_HELP))?;
        }
        cli.raise_help()?;
        cli.lower_help();
        cli.help(Help::with(help::VERSION).flag("version"))?;
        cli.raise_help()?;
        cli.lower_help();
        cli.help(Help::with(help::SHORT_HELP))?;
        Ok(Self {
            verbose: verbose,
            force: cli.check(Arg::flag("force"))?,
            variables: cli
                .get_all(Arg::option("variable").switch('v').value("key=value"))?
                .unwrap_or_default(),
            src: cli.require(Arg::positional("src"))?,
            dest: cli.require(Arg::positional("dest"))?,
        })
    }

    fn execute(self) -> proc::Result {
        // build the variable map
        let vars = VarMap::from(&self.variables);
        self.run(&vars)
    }
}

impl Koopa {
    fn run(&self, vars: &VarMap) -> Result<(), AnyError> {
        // ensure the data is allowed to be moved to the destination
        Self::has_permission(&self.dest, self.force)?;
        // perform the copy operation
        let bytes_copied = Self::copy(&self.src, &self.dest, vars, self.force)?;
        if self.verbose == true {
            // provide information back to the user that the operation was a success
            println!("info: successfully koopa'ed {} bytes", bytes_copied);
        }
        Ok(())
    }

    /// Peforms the copy operation, moving bytes from `src` to `dest` while replacing
    /// any known variables with their corresponding values.
    fn copy(src: &PathBuf, dest: &PathBuf, vars: &VarMap, force: bool) -> Result<usize, AnyError> {
        // read contents from source
        let read_words = std::fs::read_to_string(&src)?;
        // translate any variables within the text
        let write_words = Self::translate(&read_words, vars).unwrap();

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
    fn translate(text: &str, vars: &VarMap) -> Result<String, usize> {
        enum State {
            Normal,
            L1,
            Replace,
            R1,
        }

        let mut result = String::with_capacity(text.len());
        let mut raw_var = String::new();
        let mut state = State::Normal;

        let mut stream = text.char_indices();
        while let Some((_, c)) = stream.next() {
            // state transitionas
            match state {
                State::Normal => {
                    result.push(c);
                    if c == '{' {
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
                    raw_var.push(c);
                    if c == '}' {
                        state = State::R1
                    }
                }
                State::R1 => match c {
                    '}' => {
                        raw_var.pop();
                        // replace the variable with its value
                        match vars.get(&raw_var) {
                            Some(v) => result.push_str(v.as_ref()),
                            None => result.push_str(&format!("{{{{{}}}}}", raw_var)),
                        }
                        // clean up the contents stored in the variable
                        raw_var.clear();
                        state = State::Normal;
                    }
                    _ => {
                        raw_var.push(c);
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
    fn ut_translate_text() {
        let text = "hello {{ koopa.foo }} and {{ koopa.bar }}!";
        let mut vars = VarMap::new();
        vars.insert("foo", "world");
        assert_eq!(
            Koopa::translate(text, &vars).unwrap(),
            "hello world and {{ koopa.bar }}!"
        );
    }
}
