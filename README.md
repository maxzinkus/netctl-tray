# netctl-tray
A lightweight netctl tray app with notifications written in Rust.

## Usage

To launch the tray app:
```
$ netctl-tray
```
Optionally specify an update timer in milliseconds (default 2000ms):
```
$ netctl-tray 5000
```

## Installation

Note: if you use `netctl-auto`, add `--features auto` to the cargo build command

```
$ cargo build --release # --features auto if netctl-auto is used
$ sudo ./install.sh
```

## Troubleshooting

If connection strength can't be determined (`failed to read /etc/netctl/...` or
similar), ensure that the profile files in `/etc/netctl/` are readable by the
tray process. The easiest way to do this is to
`sudo chown root:<group> <profile>` where `<group>` is a group the user running
`netctl-tray` is in, and then `sudo chmod g+r <profile>`.

`iwconfig` must also be installed.

On GNOME >= 3.26 you may need to install/enable the App Indicators extension.
