# `koopa`

[![test](https://github.com/cdotrus/koopa/actions/workflows/test.yml/badge.svg?branch=trunk)](https://github.com/cdotrus/koopa/actions/workflows/test.yml) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A copy/paste tool with superpowers.

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