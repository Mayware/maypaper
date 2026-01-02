# Maypaper
*A webpage as a wallpaper? All local? How preposterous* 


https://github.com/user-attachments/assets/e5fda2fb-4649-4fdb-ab92-9d1d9f168a0f

*a demo with shadertoys, and webgl fluid sim*


## Introduction
Maypaper is a tool which allows you to put a webpage (ideally locally installed, or remote) as your wallpaper, using QT's webengine (chromium). If local, it automatically starts a webserver to serve that page, and spins it down as required.

## Usage
Maypaper includes 3 binaries, `maypaper`, `mypctl` and `myptmp`.  
Please see the [wiki](https://github.com/Mayware/maypaper/wiki/mypctl), to see how to use the program

### maypaper
`maypaper` is the main binary, which runs the wallpaper program. Run `maypaper --help` to get a list of all arguments. `maypaper` is configured purely over IPC, or by arguments and has no direct configuration file.

### mypctl
`mypctl` is the control tool for maypaper, that communicates with it over IPC. Run `mypctl --help` to get a list of all arguments.

### mytmp
`myptmp` generates local websites for you, based on templates and arguments you give it. Run `myptmp --help` to get a list of all arguments.

## Packages
| Repo | Source |
| :--: | :--: |
| Arch User Repository | [Link](https://aur.archlinux.org/packages/maypaper) |

##  Licensing
The projects's source code is licensed under `AGPL-3.0-or-later`  

The branding (eg. project name, logos etc.) is not covered by the aforementioned license, and remains the sole property of initMayday. Please seek permission from myself before using it, if required, to determine if it is an acceptable use case. Reasonable descriptive use (eg. packaging, articles, etc.) is an example of an acceptable use case. If there are any queries regarding this, please ask.  

You can purchase the program for 5GBP (or equivalent) [here](https://github.com/initMayday/licensing/blob/master/payment.md)
