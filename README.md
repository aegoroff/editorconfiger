[![CI](https://github.com/aegoroff/solt/actions/workflows/ci.yml/badge.svg)](https://github.com/aegoroff/solt/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/aegoroff/editorconfiger/branch/master/graph/badge.svg?token=FRCMPWlPN5)](https://codecov.io/gh/aegoroff/editorconfiger)
[![](https://tokei.rs/b1/github/aegoroff/editorconfiger?category=code)](https://github.com/XAMPPRocky/tokei)

# editorconfiger
`editorconfiger` is the plain tool to validate and compare .editorconfig files

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
**scoop**:

```sh
scoop bucket add aegoroff https://github.com/aegoroff/scoop-bucket.git
scoop install editorconfiger
```

**manually**:

Download the pre-compiled binaries from the [releases](https://github.com/aegoroff/editorconfiger/releases) and
copy to the desired location. RPM and DEB packages are available to install under RedHat or Debian based Linux distros.

## Command line syntax:
```
Usage: editorconfiger [COMMAND]

Commands:
  vf          Validate single .editorconfig file
  vd          Validate all found .editorconfig files in a directory and all its children
  c           Compare two .editorconfig files
  completion  Generate the autocompletion script for the specified shell
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```
