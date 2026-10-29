# Czaty — native window for gomuks web

🇵🇱 [Polska wersja](README.pl.md)

A lightweight Linux desktop window (Rust + GTK3 + WebKitGTK) for
[gomuks](https://github.com/tulir/gomuks) web — a Matrix client running on
your own server. Gomuks with bridges (e.g. WhatsApp, Signal, SMS, LinkedIn) stays
connected around the clock; this window is just a convenient viewer with a tray icon.

## Features
- WebKitGTK window with gomuks web, custom styling (`src/styl.css`, black + gold) and Polish UI strings (`src/polski.js`)
- automatic gomuks login (HTTP Basic) using credentials from a config file
- tray icon (StatusNotifierItem); closing the window hides it to the tray
- desktop notifications for new messages, persistent cookies (sign in once and stay signed in)
- `--hidden` — start hidden (for autostart), `--instaluj` — installs the icon, a `.desktop` file and autostart

## Requirements
- a running gomuks web instance (default `http://localhost:29325`) — install it following the gomuks documentation
- Linux with GTK3 and WebKitGTK 4.1 (`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`), Rust (rustup)
- a tray that supports StatusNotifierItem (COSMIC, KDE, GNOME with the AppIndicator extension)

## Building
```sh
cargo build --release
cp target/release/czaty-okno ~/.local/bin/
czaty-okno --instaluj
```

## Configuration
File `~/.config/czaty-okno/dostep` (ideally with `600` permissions):
```
GOMUKS_URL=http://localhost:29325
GOMUKS_USER=your-login
GOMUKS_PASS=your-password
```
If `GOMUKS_URL` is empty, `http://localhost:29325` is used. For a remote
server, prefer connecting through a VPN/tunnel rather than exposing gomuks publicly.

---

**Disclaimer:** Private hobby project, built for learning and personal use. Not affiliated with the gomuks project, Beeper, WhatsApp, Signal, LinkedIn or Matrix.org. Built with open-source tools. Every user signs in with their own account and is responsible for complying with the terms of the services they use. Provided “as is”, without warranty of any kind.

License: MIT — see [LICENSE](LICENSE).
