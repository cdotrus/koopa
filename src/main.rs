use cliproc::{Cli, ExitCode};
use koopa::Koopa;
use std::env;

fn main() -> ExitCode {
    Cli::default().parse(env::args()).go::<Koopa>()
}
