# Czaty — okno dla gomuks web

Lekkie okno na pulpit Linuksa (Rust + GTK3 + WebKitGTK) z interfejsem
[gomuks](https://github.com/tulir/gomuks) web — klienta Matrix działającego na
Twoim serwerze. Gomuks z mostkami (np. WhatsApp, Signal, SMS, LinkedIn) trzyma
połączenie całą dobę, a okno jest tylko wygodnym podglądem z ikoną w zasobniku.

## Funkcje
- okno WebKitGTK z gomuks web, własny wygląd (`src/styl.css`, czerń + złoto) i polskie napisy (`src/polski.js`)
- automatyczne logowanie do gomuksa (HTTP Basic) danymi z pliku konfiguracyjnego
- ikona w zasobniku (StatusNotifierItem), zamknięcie okna chowa je do zasobnika
- powiadomienia systemowe o nowych wiadomościach, trwałe ciasteczka (raz zalogowany — zostaje)
- `--hidden` — start schowany (do autostartu), `--instaluj` — ikona, plik `.desktop` i autostart

## Wymagania
- działający gomuks web (domyślnie `http://localhost:29325`) — instalacja wg dokumentacji gomuks
- Linux z GTK3 i WebKitGTK 4.1 (`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`), Rust (rustup)
- zasobnik obsługujący StatusNotifierItem (COSMIC, KDE, GNOME z rozszerzeniem AppIndicator)

## Budowanie
```sh
cargo build --release
cp target/release/czaty-okno ~/.local/bin/
czaty-okno --instaluj
```

## Konfiguracja
Plik `~/.config/czaty-okno/dostep` (najlepiej z prawami `600`):
```
GOMUKS_URL=http://localhost:29325
GOMUKS_USER=twoj-login
GOMUKS_PASS=twoje-haslo
```
Gdy `GOMUKS_URL` jest pusty, używany jest `http://localhost:29325`. Do zdalnego
serwera najlepiej łączyć się przez VPN/tunel, nie wystawiać gomuksa publicznie.

---

## English

A lightweight Linux desktop window (Rust + GTK3 + WebKitGTK) for
[gomuks](https://github.com/tulir/gomuks) web — a Matrix client running on your
own server. Gomuks with bridges (e.g. WhatsApp, Signal, SMS, LinkedIn) stays
connected 24/7; this window is just a convenient viewer with a tray icon.

### Features
- WebKitGTK window with gomuks web, custom styling (`src/styl.css`) and Polish UI strings (`src/polski.js`)
- automatic gomuks login (HTTP Basic) from a config file
- tray icon (StatusNotifierItem); closing the window hides it to the tray
- desktop notifications, persistent cookies
- `--hidden` (start hidden, for autostart), `--instaluj` (installs icon, `.desktop` file and autostart)

### Requirements
- a running gomuks web instance (default `http://localhost:29325`)
- Linux with GTK3 and WebKitGTK 4.1 (`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`), Rust
- a tray supporting StatusNotifierItem

### Building
See above (`cargo build --release`).

### Configuration
`~/.config/czaty-okno/dostep` with `GOMUKS_URL`, `GOMUKS_USER`, `GOMUKS_PASS` (chmod 600).
Prefer reaching a remote server through a VPN/tunnel instead of exposing gomuks publicly.

---

**Zastrzeżenie:** Prywatny projekt hobbystyczny, tworzony do nauki i własnego użytku. Nie jest powiązany z projektem gomuks, Beeper, WhatsApp, Signal, LinkedIn ani Matrix.org. Korzysta z otwartych narzędzi. Każdy użytkownik loguje się własnym kontem i sam odpowiada za zgodność z regulaminami używanych serwisów. Oprogramowanie udostępniane „tak jak jest”, bez gwarancji.

**Disclaimer:** Private hobby project, built for learning and personal use. Not affiliated with the gomuks project, Beeper, WhatsApp, Signal, LinkedIn or Matrix.org. Built with open-source tools. Every user signs in with their own account and is responsible for complying with the terms of the services they use. Provided “as is”, without warranty of any kind.

Licencja / License: MIT — see [LICENSE](LICENSE).
