# Contributing
Thanks for looking to contribute! Here's some stuff I think you should know, before you get started!

Maypaper uses webkitgtk, particularly, the gtk4 version. gtk-layer-shell for gtk4 is used to put the window to the background. src/main.rs produces the maypaper binary, and the other relevant binary rs files can be found in /src/bin.

Also note, maypaper sets GSK_RENDERER = gl. This can be overriden by the user by specifying an alternative, but the reason this is here is because Nvidia in particular had extreme lag when it was not set. 

Here's the current roadmap of architectural stuff I'd like to add:

- Introduce variable injection, to allow configuration of the webpages. For example, injecting the volume the user wants, speed, etc. Other things like the current workspace the wallpaper is on would also be cool
- Perhaps a configuration GUI, that manages wallpapers.toml for you? I personally wouldn't use or require this, but for newer linux users it would be helpful.
