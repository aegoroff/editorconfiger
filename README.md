[![CI](https://github.com/aegoroff/solt/actions/workflows/ci.yml/badge.svg)](https://github.com/aegoroff/solt/actions/workflows/ci.yml)

# editorconfiger
Plain tool to validate and compare .editorconfig files

## Install the pre-compiled binary

**homebrew** (only on macOS and Linux for now):

Add my tap (do it once):
```sh
brew tap aegoroff/tap
```
And then install editorconfiger:
```sh
brew install editorconfiger
```
Update editorconfiger if already installed:
```sh
brew upgrade editorconfiger
```
## Command line syntax:
```
USAGE:
    editorconfig [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    c       Compare two .editorconfig files
    help    Prints this message or the help of the given subcommand(s)
    vd      Validate all found .editorconfig files in a directory and all its children
    vf      Validate single .editorconfig file
```