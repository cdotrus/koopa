pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const SHORT_HELP: &str = "\
Koopa is a copy/paste tool with superpowers.

Usage:
    kp [options] <src> <dest>

Arguments:
    <src>           filesystem path to copy
    <dest>          filesystem path to place copied contents 

Options:
    --shell, -s <key=value>...  specify runtime variables 
    --force                     bypass safety checks and errors
    --verbose                   use verbose output
    --version                   print version information and exit
    --help, -h                  print this help information and exit

Use 'kp --help --verbose' for more information about koopa.
";

pub const LONG_HELP: &str = "\
Koopa is a copy/paste tool with superpowers.
";
