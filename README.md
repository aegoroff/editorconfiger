[![CI](https://github.com/aegoroff/editorconfiger/actions/workflows/ci.yml/badge.svg)](https://github.com/aegoroff/editorconfiger/actions/workflows/ci.yml)
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

**AUR (Arch Linux User Repository)**:

install binary package:
```sh
 yay -S editorconfiger-bin
```
or if yay reports that package not found force updating repo info
```sh
yay -Syyu editorconfiger-bin
```
install using cargo so builiding on target machine:
```sh
 yay -S editorconfiger
```
or if yay reports that package not found force updating repo info
```sh
yay -Syyu editorconfiger
```


**manually**:

Download the pre-compiled binaries from the [releases](https://github.com/aegoroff/editorconfiger/releases) and
copy to the desired location. RPM and DEB packages are available to install under RedHat or Debian based Linux distros.

**install deb package on Arch Linux**:

1. Install [debtap](https://github.com/helixarch/debtap) from AUR using yay:
```sh
 yay -S debtap
```
2. Create equivalent package using debtap:
```sh
 sudo debtap -u
 debtap editorconfiger_x.x.x_amd64.deb
 ```
3. Install using pacman:
```sh
sudo pacman -U editorconfiger-x.x.x-1-x86_64.pkg.tar.zst
```

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
