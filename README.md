# Maypaper
The (upcoming) dynamic wallpaper tool!

## Introduction
Maypaper is a tool which allows you to put a webpage (ideally locally installed, or remote) as your wallpaper, using webkitgtk. It comes with accompanying tools to manage the installation and updates of wallpapers, although these are entirely optional.

## Usage
Maypaper includes 3 binaries, `maypaper`, `mypctl` and `myppm`.

### maypaper
`maypaper` is the main binary, which runs the wallpaper program. Run `maypaper --help` to get a list of all arguments. `maypaper` is configured purely over IPC, or by arguments and has no direct configuration file.

### mypctl
`mypctl` is the control tool for maypaper, that communicates with it over IPC. Run `mypctl --help` to get a list of all arguments.

### myppm
`myppm` is the package manager for maypaper, in the sense that it manages wallpapers for you. It does a partial clones of git repositories (you specify the folders you want), and keeps them updated. Run `myppm --help` to get a list of all arguments. Upon the first run of `myppm`, a configuration directory will be created at `$XDG_RUNTIME_DIR/maypaper`, where a file called `wallpapers.toml` will exist. Within there, you can specify the wallpapers you wish to be declaratively managed by `myppm`.

## General
You can make your own repository for wallpapers, use someone else's, or alternatively, here exists a repository under my control which may be of use to you: {}. Just to reinforce, it is completely decentralised, `myppm` makes no assumptions about what repository you are using.

## Packages

## Contributing
Thanks for considering to contribute! Please read [Contributing.md](Contributing.md) to get an overview of the project, goals, and other information!

##  Licensing
The projects's source code is licensed under `AGPL-3.0-or-later`  

The branding (eg. project name, logos etc.) is not covered by the aforementioned license, and remains the sole property of initMayday. Please seek permission from myself before using it, if required, to determine if it is an acceptable use case. Reasonable descriptive use (eg. packaging, articles, etc.) is an example of an acceptable use case. If there are any queries regarding this, please ask.  

You can purchase the program for 5GBP (or equivalent) [here](https://github.com/initMayday/licensing/blob/master/payment.md)
