//! Okno „Czaty” — lekka nakładka na gomuks web (klient Matrix) działający na Twoim
//! serwerze (domyślnie `http://localhost:29325`), który trzyma połączenie z mostkami
//! (np. WhatsApp, Signal, SMS, LinkedIn) całą dobę. Okno jest tylko podglądem.
//!
//! Wygląd: `styl.css` (czerń + złoto) wstrzykiwany do strony.
//! Logowanie do gomuksa samo — dane z `~/.config/czaty-okno/dostep` (GOMUKS_URL/USER/PASS).
//!
//! X chowa okno do zasobnika (powiadomienia dalej przychodzą); Ctrl+Q albo menu ikonki zamyka.
//! `czaty-okno --hidden` — start schowany (autostart). `czaty-okno --instaluj` — ikona,
//! wpis w menu i autostart. Drugie uruchomienie (skrót Super+;) pokazuje/chowa okno.
use gtk::glib;
use gtk::prelude::*;
use ksni::menu::StandardItem;
use javascriptcore::ValueExt;
use std::sync::Mutex;
use ksni::{MenuItem, ToolTip, Tray, TrayMethods};
use std::cell::RefCell;
use std::rc::Rc;
use webkit2gtk::{
    AuthenticationRequestExt, CookieManagerExt, DownloadExt, CookiePersistentStorage, Credential, CredentialPersistence, LoadEvent,
    NotificationExt, NotificationPermissionRequest, PermissionRequestExt, SettingsExt, UserContentInjectedFrames,
    UserContentManager, UserContentManagerExt, UserStyleLevel, UserStyleSheet, WebContext, WebContextExt, WebView,
    WebViewExt, WebsiteDataManager, WebsiteDataManagerExt,
};

/// Nazwa pliku z atrybutu `download` linku (adres mediów gomuksa jej nie zawiera).
static NAZWA_POBRANIA: Mutex<Option<String>> = Mutex::new(None);
const POBIERANIE_JS: &str = r#"document.addEventListener('click', function (e) {
  var a = e.target.closest && e.target.closest('a[href]');
  if (!a || !/\/_gomuks\/media\//.test(a.href) || !(a.hasAttribute('download') || a.target === '_blank')) return;
  e.preventDefault(); e.stopPropagation();
  window.webkit.messageHandlers.pobierz.postMessage(a.href + '\n' + (a.getAttribute('download') || ''));
}, true);"#;

const APP_ID: &str = "io.github.longer54.Czaty";
const STYL: &str = include_str!("styl.css");
const POLSKI: &str = include_str!("polski.js"); // spolszczenie interfejsu gomuksa
const IKONY: [(u32, &[u8]); 8] = [
    (16, include_bytes!("../ikona/czaty-16.png")),
    (22, include_bytes!("../ikona/czaty-22.png")),
    (24, include_bytes!("../ikona/czaty-24.png")),
    (32, include_bytes!("../ikona/czaty-32.png")),
    (48, include_bytes!("../ikona/czaty-48.png")),
    (64, include_bytes!("../ikona/czaty-64.png")),
    (128, include_bytes!("../ikona/czaty-128.png")),
    (256, include_bytes!("../ikona/czaty-256.png")),
];
// Pasek okna w kolorach Agentów zamiast szarego GTK.
const CSS_OKNA: &str = "
headerbar { min-height: 30px; padding: 0 6px; background: #111111; border: 0; box-shadow: none; }
headerbar .title { color: #d4af37; font-weight: 600; }
headerbar button { color: #9a9a9a; background: transparent; border: 0; box-shadow: none; min-height: 22px; min-width: 22px; }
headerbar button:hover { color: #d4af37; background: #262626; }
decoration { border-radius: 0; box-shadow: 0 0 0 1px #2a2a2a, 0 6px 22px rgba(0,0,0,.6); }
window.csd, window.csd > * { border-radius: 0; }
";
// WebKitGTK nie oddaje stronie obrazków ze schowka — okno czyta je samo i podaje tu jako PNG,
// a ten skrypt udaje zwykłe wklejenie w polu wiadomości (gomuks otwiera okienko wysyłki pliku).
const WKLEJ_JS: &str = "
window.czatyWklejObraz = (b64) => {
  const bajty = Uint8Array.from(atob(b64), c => c.charCodeAt(0));
  const plik = new File([bajty], 'obraz.png', { type: 'image/png' });
  const dt = new DataTransfer();
  dt.items.add(plik);
  const cel = document.querySelector('.message-composer textarea') || document.activeElement || document.body;
  cel.dispatchEvent(new ClipboardEvent('paste', { clipboardData: dt, bubbles: true, cancelable: true }));
};
";

// gomuks domyślnie pokazuje „ukryte” zdarzenia — u Beepera to setki `com.beeper.message_send_status`
// (potwierdzenia doręczenia). v26.09 nie bierze domyślnych ustawień z serwera, więc ustawiamy je
// w localStorage przed startem strony (tylko jeśli użytkownik sam tego nie przestawił).
const USTAWIENIA_JS: &str = "
try {
  const p = JSON.parse(localStorage.getItem('global_prefs') || '{}');
  if (p.show_hidden_events === undefined) {
    p.show_hidden_events = false;
    localStorage.setItem('global_prefs', JSON.stringify(p));
  }
} catch (e) {}
// Powiadomienia: gomuks prosi o zgodę tylko z przycisku w ustawieniach — prosimy od razu,
// a okno (connect_permission_request) zgadza się samo.
if (window.Notification && Notification.permission !== 'granted') {
  document.addEventListener('DOMContentLoaded', () => Notification.requestPermission());
}
";

struct Dostep {
    url: String,
    user: String,
    pass: String,
}

/// Domyślny adres gomuks web, gdy w pliku `dostep` nie ma `GOMUKS_URL`.
fn adres_domyslny() -> String {
    "http://localhost:29325".into()
}

fn dostep() -> Dostep {
    let plik = glib::user_config_dir().join("czaty-okno/dostep");
    let tresc = std::fs::read_to_string(&plik).unwrap_or_default();
    let pole = |k: &str| {
        tresc.lines().find_map(|l| l.strip_prefix(&format!("{k}="))).unwrap_or_default().trim().to_string()
    };
    let url = pole("GOMUKS_URL");
    Dostep {
        url: if url.is_empty() { adres_domyslny() } else { url },
        user: pole("GOMUKS_USER"),
        pass: pole("GOMUKS_PASS"),
    }
}

fn instaluj() -> std::io::Result<()> {
    let dane = glib::user_data_dir();
    for (r, png) in IKONY {
        let kat = dane.join(format!("icons/hicolor/{r}x{r}/apps"));
        std::fs::create_dir_all(&kat)?;
        std::fs::write(kat.join("czaty.png"), png)?;
    }
    let bin = glib::home_dir().join(".local/bin/czaty-okno");
    let wpis = |exec: String| {
        format!(
            "[Desktop Entry]\nType=Application\nName=Czaty\nComment=WhatsApp, Signal, SMS i LinkedIn — lekko, przez gomuks\n\
             Exec={exec}\nIcon=czaty\nCategories=Network;Chat;\nStartupWMClass={APP_ID}\nTerminal=false\n"
        )
    };
    let apps = dane.join("applications");
    std::fs::create_dir_all(&apps)?;
    std::fs::write(apps.join("czaty.desktop"), wpis(bin.display().to_string()))?;
    let auto = glib::user_config_dir().join("autostart");
    std::fs::create_dir_all(&auto)?;
    std::fs::write(auto.join("czaty.desktop"), wpis(format!("{} --hidden", bin.display())))?;
    let _ = std::process::Command::new("gtk-update-icon-cache")
        .args(["-q", "-t", &dane.join("icons/hicolor").to_string_lossy()])
        .status();
    Ok(())
}

enum Polecenie {
    Pokaz,
    PrzelaczWidok,
    Zamknij,
}

struct Zasobnik {
    tx: glib::Sender<Polecenie>,
}

impl Tray for Zasobnik {
    fn id(&self) -> String {
        "czaty".into()
    }
    fn icon_name(&self) -> String {
        "czaty".into()
    }
    fn title(&self) -> String {
        "Czaty".into()
    }
    fn tool_tip(&self) -> ToolTip {
        ToolTip { title: "Czaty".into(), description: "WhatsApp, Signal, SMS, LinkedIn".into(), ..Default::default() }
    }
    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx.send(Polecenie::PrzelaczWidok);
    }
    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            StandardItem {
                label: "Otwórz czaty".into(),
                activate: Box::new(|t: &mut Self| {
                    let _ = t.tx.send(Polecenie::Pokaz);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Zamknij aplikację".into(),
                activate: Box::new(|t: &mut Self| {
                    let _ = t.tx.send(Polecenie::Zamknij);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

fn uruchom_zasobnik(tx: glib::Sender<Polecenie>) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("tokio");
        rt.block_on(async move {
            loop {
                match (Zasobnik { tx: tx.clone() }).spawn().await {
                    Ok(uchwyt) => {
                        std::future::pending::<()>().await;
                        drop(uchwyt);
                    }
                    Err(e) => {
                        eprintln!("czaty-okno: zasobnik niedostępny ({e}), ponawiam za 2 s");
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                }
            }
        });
    });
}

fn build_window(app: &gtk::Application, tx: glib::Sender<Polecenie>) -> gtk::ApplicationWindow {
    let d = dostep();
    // Trwałe ciasteczka — gomuks pamięta zalogowanie, a sesja mostków i tak żyje na serwerze.
    let dane = glib::user_data_dir().join("czaty-okno");
    let cache = glib::user_cache_dir().join("czaty-okno");
    let wdm = WebsiteDataManager::builder()
        .base_data_directory(dane.to_string_lossy().as_ref())
        .base_cache_directory(cache.to_string_lossy().as_ref())
        .build();
    if let Some(ciastka) = wdm.cookie_manager() {
        ciastka.set_persistent_storage(&dane.join("ciasteczka.sqlite").to_string_lossy(), CookiePersistentStorage::Sqlite);
    }
    let kontekst = WebContext::with_website_data_manager(&wdm);
    // Pobrania → ~/Pobrane (bez nadpisywania istniejących) + powiadomienie.
    kontekst.connect_download_started(|_, dl| {
        dl.connect_decide_destination(|dl, sugerowana| {
            let kat = glib::home_dir().join("Pobrane");
            let _ = std::fs::create_dir_all(&kat);
            let wybrana = NAZWA_POBRANIA.lock().ok().and_then(|mut n| n.take()).filter(|n| !n.is_empty());
            let czysta = std::path::Path::new(wybrana.as_deref().unwrap_or(sugerowana))
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| "plik".into());
            let (trzon, roz) = match czysta.rsplit_once('.') {
                Some((a, b)) if !a.is_empty() => (a.to_string(), format!(".{b}")),
                _ => (czysta.clone(), String::new()),
            };
            let mut cel = kat.join(&czysta);
            let mut i = 1;
            while cel.exists() {
                cel = kat.join(format!("{trzon}-{i}{roz}"));
                i += 1;
            }
            dl.set_destination(&format!("file://{}", cel.display()));
            true
        });
        dl.connect_finished(|dl| {
            if let Some(d) = dl.destination() {
                let sciezka = d.trim_start_matches("file://").to_string();
                let _ = std::process::Command::new("notify-send")
                    .args(["-i", "folder-download", "Czaty — pobrano", &sciezka])
                    .spawn();
            }
        });
    });
    kontekst.set_cache_model(webkit2gtk::CacheModel::DocumentViewer);

    let ucm = UserContentManager::new();
    ucm.add_style_sheet(&UserStyleSheet::new(STYL, UserContentInjectedFrames::TopFrame, UserStyleLevel::User, &[], &[]));
    if std::env::var_os("CZATY_DEBUG").is_some() {
        ucm.add_script(&webkit2gtk::UserScript::new(
            "window.czatyDebug = true;",
            UserContentInjectedFrames::TopFrame,
            webkit2gtk::UserScriptInjectionTime::Start,
            &[],
            &[],
        ));
    }
    ucm.add_script(&webkit2gtk::UserScript::new(
        POLSKI,
        UserContentInjectedFrames::TopFrame,
        webkit2gtk::UserScriptInjectionTime::End,
        &[],
        &[],
    ));
    ucm.add_script(&webkit2gtk::UserScript::new(
        WKLEJ_JS,
        UserContentInjectedFrames::TopFrame,
        webkit2gtk::UserScriptInjectionTime::End,
        &[],
        &[],
    ));
    ucm.add_script(&webkit2gtk::UserScript::new(
        USTAWIENIA_JS,
        UserContentInjectedFrames::TopFrame,
        webkit2gtk::UserScriptInjectionTime::Start,
        &[],
        &[],
    ));
    ucm.register_script_message_handler("pobierz");
    ucm.add_script(&webkit2gtk::UserScript::new(
        POBIERANIE_JS,
        UserContentInjectedFrames::TopFrame,
        webkit2gtk::UserScriptInjectionTime::End,
        &[],
        &[],
    ));
    let webview = WebView::builder().web_context(&kontekst).user_content_manager(&ucm).build();
    {
        let wv = webview.clone();
        ucm.connect_script_message_received(Some("pobierz"), move |_, wynik| {
            let tresc = wynik.js_value().map(|v| v.to_str().to_string()).unwrap_or_default();
            let (adres, nazwa) = tresc.split_once('\n').unwrap_or((tresc.as_str(), ""));
            if let Ok(mut n) = NAZWA_POBRANIA.lock() {
                *n = Some(nazwa.to_string());
            }
            wv.download_uri(adres);
        });
    }
    if let Some(ust) = WebViewExt::settings(&webview) {
        ust.set_enable_developer_extras(false);
        // CZATY_DEBUG=1 czaty-okno → konsola strony na stdout + liczniki tłumacza
        ust.set_enable_write_console_messages_to_stdout(std::env::var_os("CZATY_DEBUG").is_some());
    }

    // Pytanie gomuksa o login → podajemy sami (Basic auth), bez okienka.
    let (user, pass) = (d.user.clone(), d.pass.clone());
    webview.connect_authenticate(move |_, req| {
        if user.is_empty() || req.is_retry() {
            return false; // brak danych / złe hasło → zwykłe okienko WebKita
        }
        let c = Credential::new(&user, &pass, CredentialPersistence::ForSession);
        // webkit_authentication_request_authenticate nie ma bezpiecznego wrappera w webkit2gtk-rs 2.0
        use gtk::glib::translate::ToGlibPtr;
        unsafe {
            let cp: *const webkit2gtk::ffi::WebKitCredential = c.to_glib_none().0;
            webkit2gtk::ffi::webkit_authentication_request_authenticate(req.to_glib_none().0, cp as *mut _);
        }
        true
    });

    // Powiadomienia o nowych wiadomościach — zgoda bez pytania (strona jest nasza).
    webview.connect_permission_request(|_, req| {
        if req.downcast_ref::<NotificationPermissionRequest>().is_some() {
            req.allow();
            return true;
        }
        false
    });
    // Klik w powiadomienie → okno na wierzch (gomuks sam przełącza na tę rozmowę).
    let tx_powiad = tx.clone();
    webview.connect_show_notification(move |_, n| {
        let tx = tx_powiad.clone();
        n.connect_clicked(move |_| {
            let _ = tx.send(Polecenie::Pokaz);
        });
        false
    });

    let adres = d.url.clone();
    webview.load_uri(&adres);
    // Serwer nie odpowiada (brak sieci / VPN wstaje) → próbuj co 3 s.
    let adres2 = adres.clone();
    webview.connect_load_failed(move |wv, _ev: LoadEvent, uri, _err| {
        if !uri.starts_with(&adres2) {
            return false;
        }
        let wv = wv.clone();
        let uri = uri.to_string();
        glib::timeout_add_seconds_local_once(3, move || wv.load_uri(&uri));
        true
    });

    // Linki z wiadomości → zwykła przeglądarka.
    let adres3 = adres.clone();
    webview.connect_decide_policy(move |_, decyzja, typ| {
        use webkit2gtk::{NavigationPolicyDecision, NavigationPolicyDecisionExt, PolicyDecisionExt, PolicyDecisionType, URIRequestExt};
        // Załączniki (pliki, których WebKit nie umie pokazać, albo z nagłówkiem attachment) → pobranie do ~/Pobrane.
        if typ == PolicyDecisionType::Response {
            if let Some(rd) = decyzja.downcast_ref::<webkit2gtk::ResponsePolicyDecision>() {
                use webkit2gtk::{ResponsePolicyDecisionExt, URIResponseExt};
                let zalacznik = rd
                    .response()
                    .and_then(|r| r.http_headers())
                    .and_then(|h| h.one("Content-Disposition").map(|v| v.to_string()))
                    .map_or(false, |v| v.trim_start().to_ascii_lowercase().starts_with("attachment"));
                if zalacznik || !rd.is_mime_type_supported() {
                    decyzja.download();
                    return true;
                }
            }
            return false;
        }
        if typ != PolicyDecisionType::NavigationAction && typ != PolicyDecisionType::NewWindowAction {
            return false;
        }
        let Some(nd) = decyzja.downcast_ref::<NavigationPolicyDecision>() else { return false };
        let Some(uri) = nd.navigation_action().and_then(|a| a.request()).and_then(|r| r.uri()) else { return false };
        if uri.starts_with(&adres3) || !uri.starts_with("http") {
            return false;
        }
        let _ = gtk::gio::AppInfo::launch_default_for_uri(&uri, None::<&gtk::gio::AppLaunchContext>);
        decyzja.ignore();
        true
    });

    let window = gtk::ApplicationWindow::new(app);
    window.set_title("Czaty");
    window.set_icon_name(Some("czaty"));
    window.set_default_size(1100, 780);
    let pasek = gtk::HeaderBar::new();
    pasek.set_title(Some("Czaty"));
    pasek.set_show_close_button(true);
    pasek.set_decoration_layout(Some(":minimize,maximize,close"));
    window.set_titlebar(Some(&pasek));
    let css = gtk::CssProvider::new();
    if css.load_from_data(CSS_OKNA.as_bytes()).is_ok() {
        if let Some(ekran) = gtk::prelude::WidgetExt::screen(&window) {
            gtk::StyleContext::add_provider_for_screen(&ekran, &css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        }
    }
    window.add(&webview);

    // gomuks pokazuje powiadomienie tylko, gdy strona „nie ma fokusu” (window focus/blur).
    // Schowanie okna do zasobnika nie wysyła stronie blur → myślała, że użytkownik patrzy, i milczała.
    // Fokus strony = aktywność okna GTK.
    let wv = webview.clone();
    window.connect_is_active_notify(move |win| {
        let js = if win.is_active() {
            "window.dispatchEvent(new Event('focus'))"
        } else {
            "window.dispatchEvent(new Event('blur'))"
        };
        wv.run_javascript(js, None::<&gtk::gio::Cancellable>, |_| {});
    });

    // X = schowaj (proces żyje, powiadomienia lecą dalej).
    window.connect_delete_event(|win, _| {
        win.hide();
        glib::Propagation::Stop
    });
    let wv = webview.clone();
    window.connect_key_press_event(move |win, ev| {
        use gtk::gdk::keys::constants as key;
        if ev.keyval() == key::Escape && ev.state().is_empty() {
            return glib::Propagation::Proceed;
        }
        if !ev.state().contains(gtk::gdk::ModifierType::CONTROL_MASK) {
            return glib::Propagation::Proceed;
        }
        match ev.keyval() {
            key::r | key::R => {
                wv.reload_bypass_cache();
                glib::Propagation::Stop
            }
            key::v | key::V => {
                let schowek = gtk::Clipboard::get(&gtk::gdk::SELECTION_CLIPBOARD);
                if schowek.wait_is_text_available() || !schowek.wait_is_image_available() {
                    return glib::Propagation::Proceed;
                }
                let Some(obraz) = schowek.wait_for_image() else { return glib::Propagation::Proceed };
                let Ok(png) = obraz.save_to_bufferv("png", &[]) else { return glib::Propagation::Proceed };
                let js = format!("window.czatyWklejObraz && window.czatyWklejObraz('{}')", glib::base64_encode(&png));
                wv.run_javascript(&js, None::<&gtk::gio::Cancellable>, |_| {});
                glib::Propagation::Stop
            }
            key::q | key::Q => {
                if let Some(app) = win.application() {
                    app.quit();
                }
                glib::Propagation::Stop
            }
            key::w | key::W => {
                win.hide();
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        }
    });
    window
}

fn main() {
    let argumenty: Vec<String> = std::env::args().skip(1).collect();
    if argumenty.iter().any(|a| a == "--instaluj") {
        if let Err(e) = instaluj() {
            eprintln!("czaty-okno --instaluj: {e}");
            std::process::exit(1);
        }
        return;
    }
    let schowany = Rc::new(std::cell::Cell::new(argumenty.iter().any(|a| a == "--hidden")));
    glib::set_prgname(Some(APP_ID));
    let app = gtk::Application::new(Some(APP_ID), gtk::gio::ApplicationFlags::empty());
    let window: Rc<RefCell<Option<gtk::ApplicationWindow>>> = Rc::new(RefCell::new(None));
    let nadajnik: Rc<RefCell<Option<glib::Sender<Polecenie>>>> = Rc::new(RefCell::new(None));
    app.connect_activate(move |app| {
        // Kolejne uruchomienie (Super+; albo ikona w menu) → pokaż albo schowaj.
        if window.borrow().is_some() {
            if let Some(tx) = nadajnik.borrow().as_ref() {
                let _ = tx.send(Polecenie::PrzelaczWidok);
            }
            return;
        }
        let (tx, rx) = glib::MainContext::channel::<Polecenie>(glib::Priority::DEFAULT);
        let win = build_window(app, tx.clone());
        std::mem::forget(app.hold()); // schowane okno nie zamyka appki
        let (w, a) = (win.clone(), app.clone());
        rx.attach(None, move |p| {
            match p {
                Polecenie::Pokaz => {
                    w.show_all();
                    w.present();
                }
                Polecenie::PrzelaczWidok => {
                    if w.is_visible() && w.is_active() {
                        w.hide();
                    } else {
                        w.show_all();
                        w.present();
                    }
                }
                Polecenie::Zamknij => a.quit(),
            }
            glib::ControlFlow::Continue
        });
        *nadajnik.borrow_mut() = Some(tx.clone());
        uruchom_zasobnik(tx);
        if schowany.replace(false) {
            win.realize();
        } else {
            win.show_all();
            win.present();
        }
        *window.borrow_mut() = Some(win);
    });
    app.run_with_args::<&str>(&[]);
}
