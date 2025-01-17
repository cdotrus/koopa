# `koopa`

[![Test](https://github.com/chaseruskin/koopa/actions/workflows/test.yml/badge.svg?branch=trunk)](https://github.com/chaseruskin/koopa/actions/workflows/test.yml) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A copy/paste tool with superpowers.

## Install

Run the following Cargo command:
```
cargo install --git https://github.com/chaseruskin/koopa
```

## Usage

```
Koopa is a copy/paste tool with superpowers.

Usage:
    kp [options] <src> <dest>

Arguments:
    <src>           filesystem path to copy
    <dest>          filesystem path to place copied contents 

Options:
    --shell, -s <key=value>...  specify runtime in-line text replacements
    --ignore-work               ignore .koopa folders along the working path
    --ignore-home               ignore the .koopa folder at the home path
    --force                     bypass safety checks and errors
    --verbose                   use verbose output
    --list                      list available files + shells and exit
    --version                   print version information and exit
    --help, -h                  print this help information and exit

Use 'kp --help --verbose' for more information about koopa.
```

## Details

At its core, koopa is used to copy files and directories across your filesystem. Koopa builds on top of the simple copy operation through _shells_ and _sources_.

A _shell_ is a key-value pair. You can place shells in any text file for koopa to find and replace during the copy operation.

A _source_ is a regular text file you wish to copy, which may or may not have any shells defined. These are essentially your templates you wish to reuse across projects and different contexts.