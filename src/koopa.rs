use super::error::Error;
use super::help;
use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Command, Help};
use std::path::PathBuf;

type AnyError = Box<dyn std::error::Error>;

#[derive(Debug, PartialEq)]
pub struct Koopa {
    src: PathBuf,
    dest: PathBuf,
    force: bool,
    verbose: bool,
    variables: Vec<String>,
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
        self.copy()
    }
}

impl Koopa {
    fn copy(&self) -> Result<(), AnyError> {
        // ensure the data is allowed to be moved to the destination
        self.has_permission()?;
        // perform the copy operation
        let bytes_copied = std::fs::copy(&self.src, &self.dest)?;
        if self.verbose == true {
            // provide information back to the user that the operation was a success
            println!("info: successfully copied {} bytes", bytes_copied);
        }
        Ok(())
    }

    /// Verifies the data is allowed to be placed at the destination path.
    fn has_permission(&self) -> Result<(), Error> {
        match self.force == false && self.dest.exists() == true {
            true => Err(Error::DestinationExists(self.dest.clone())),
            false => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ut_has_permission_err() {
        let kp = Koopa {
            src: PathBuf::default(),
            dest: PathBuf::from("README.md"),
            force: false,
            verbose: false,
            variables: Vec::default(),
        };
        assert_eq!(
            Error::DestinationExists(kp.dest.clone()),
            kp.has_permission().unwrap_err()
        )
    }

    #[test]
    fn ut_has_permission_ok() {
        let kp = Koopa {
            src: PathBuf::default(),
            dest: PathBuf::from("some_unnamed_file.txt"),
            force: false,
            verbose: false,
            variables: Vec::default(),
        };
        assert_eq!(Ok(()), kp.has_permission())
    }
}
