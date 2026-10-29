// Czaty — spolszczenie gomuksa (gomuks web nie ma tłumaczeń).
// Podmienia TYLKO całe napisy interfejsu (przyciski, menu, podpowiedzi, placeholdery).
// Treści wiadomości, nazw rozmów i podglądów nie dotyka (pomijamy oś czasu i listę rozmów).
(() => {
  const S = {
    // lista rozmów / pasek boczny
    "Search rooms": "Szukaj rozmów", "Search rooms to add...": "Szukaj rozmów do dodania…",
    "Create room": "Nowa rozmowa", "Create a new room": "Nowa rozmowa", "Notification Center": "Powiadomienia",
    "Notifications": "Powiadomienia", "No more notifications": "Brak powiadomień",
    "Favorite": "Ulubione", "Unfavorite": "Usuń z ulubionych", "Set low priority": "Niski priorytet",
    "Unset low priority": "Zwykły priorytet", "Leave room": "Opuść rozmowę", "Settings": "Ustawienia",
    "Share": "Udostępnij", "View space": "Pokaż przestrzeń", "Mute": "Wycisz", "Unmute": "Włącz dźwięk",
    // nagłówek rozmowy / prawy panel
    "Pinned Messages": "Przypięte", "No pinned messages": "Brak przypiętych wiadomości",
    "Room Members": "Uczestnicy", "Filter members": "Szukaj uczestników", "Room Settings": "Ustawienia rozmowy",
    "Room info": "Informacje", "Widgets in room": "Widżety", "Search": "Szukaj", "Search term": "Szukana fraza",
    "Shared rooms": "Wspólne rozmowy", "Back": "Wstecz", "Room name": "Nazwa rozmowy", "Room topic": "Opis",
    "Topic": "Opis", "Name": "Nazwa", "Devices": "Urządzenia", "Device": "Urządzenie", "Security": "Bezpieczeństwo",
    "Private chat": "Czat prywatny", "Public chat": "Czat publiczny", "Trusted private chat": "Zaufany czat prywatny",
    // pole pisania
    "Send a message": "Napisz wiadomość", "Send a message (encrypted)": "Napisz wiadomość (szyfrowana)",
    "Send message": "Wyślij", "Send": "Wyślij", "Upload attachment": "Załącz plik", "Upload": "Wyślij plik",
    "Add emoji": "Emoji", "Search emojis": "Szukaj emoji", "Search stickers": "Szukaj naklejek",
    "Send as file": "Wyślij jako plik", "Original size": "Oryginalny rozmiar", "Resize": "Zmień rozmiar",
    "Quality": "Jakość", "File name": "Nazwa pliku", "Uploading media...": "Wysyłanie…",
    "Click to show composer anyway": "Kliknij, aby mimo to pisać",
    // menu wiadomości
    "Reply": "Odpowiedz", "React": "Reakcja", "Reactions": "Reakcje", "Edit": "Edytuj", "Open thread": "Otwórz wątek",
    "Pin message": "Przypnij", "Unpin message": "Odepnij", "Remove": "Usuń", "Remove Message": "Usuń wiadomość",
    "Report": "Zgłoś", "Report Message": "Zgłoś wiadomość", "Send report": "Wyślij zgłoszenie",
    "Resend": "Wyślij ponownie", "Resend message": "Wyślij ponownie", "Undo": "Cofnij", "Undo send": "Cofnij wysłanie",
    "View source": "Pokaż źródło", "Hide content": "Ukryj treść", "View content": "Pokaż treść",
    "Request key": "Poproś o klucz", "Reason for removal": "Powód usunięcia", "Reason for report": "Powód zgłoszenia",
    "Reason (optional)": "Powód (opcjonalnie)",
    "Waiting for /send to return": "Wysyłanie…", "Waiting to receive event in /sync": "Wysłano",
    "Event sent and remote echo received": "Doręczono do serwera",
    "This is the beginning of the thread.": "To początek wątku.",
    // ogólne
    "Cancel": "Anuluj", "Save": "Zapisz", "Delete": "Usuń", "Discard": "Odrzuć", "Create": "Utwórz",
    "Reject": "Odrzuć", "Reject all": "Odrzuć wszystkie", "Actions": "Działania", "Options": "Opcje",
    "Advanced options": "Opcje zaawansowane", "Account": "Konto", "Logout": "Wyloguj", "Verify": "Zweryfikuj",
    "Close poll": "Zamknij ankietę", "Load votes": "Pokaż głosy", "Global profile": "Profil",
    "Ban": "Zablokuj", "Unban": "Odblokuj", "Invited by": "Zaproszenie od", "Timestamp": "Czas",
    "Encryption": "Szyfrowanie", "Custom CSS": "Własny CSS", "Profile biography": "O mnie",
  };
  const WZORY = [
    [/^(\w[\w ]*?) private chat with (.+)$/, "$1 — czat prywatny z $2"],
    [/^(\w[\w ]*?) group chat$/, "$1 — czat grupowy"],
    [/^(\d+) members?$/, "$1 uczestn."],
  ];
  const ATRYBUTY = ["placeholder", "title", "aria-label"];
  // tu są treści rozmów — nie tłumaczymy tekstu (atrybuty przycisków tak)
  const POMIN = ".timeline-list, .room-entry, .message-preview, .event-content, .html-body, .reply-body, textarea";

  const tlumacz = (t) => {
    const s = t.trim();
    if (!s) return null;
    if (Object.prototype.hasOwnProperty.call(S, s)) return t.replace(s, S[s]);
    for (const [re, na] of WZORY) if (re.test(s)) return t.replace(s, s.replace(re, na));
    return null;
  };
  const tekst = (n) => {
    const p = n.parentElement;
    if (!p || p.closest(POMIN)) return;
    const nowy = tlumacz(n.nodeValue);
    if (nowy !== null && nowy !== n.nodeValue) n.nodeValue = nowy;
  };
  const atrybuty = (el) => {
    for (const a of ATRYBUTY) {
      const v = el.getAttribute(a);
      if (v) {
        const nowy = tlumacz(v);
        if (nowy !== null && nowy !== v) el.setAttribute(a, nowy);
      }
    }
  };
  const wezel = (n) => {
    if (n.nodeType === 3) return tekst(n);
    if (n.nodeType !== 1) return;
    atrybuty(n);
    if (n.closest(POMIN)) {
      n.querySelectorAll("[title],[placeholder],[aria-label]").forEach(atrybuty);
      return;
    }
    const w = document.createTreeWalker(n, NodeFilter.SHOW_TEXT | NodeFilter.SHOW_ELEMENT, {
      acceptNode: (x) => (x.nodeType === 1 && x.matches(POMIN)) ? NodeFilter.FILTER_REJECT : NodeFilter.FILTER_ACCEPT,
    });
    for (let x = w.nextNode(); x; x = w.nextNode()) x.nodeType === 3 ? tekst(x) : atrybuty(x);
  };
  const start = () => {
    wezel(document.body);
    let licz = 0, zm = 0;
    if (window.czatyDebug) setInterval(() => { console.log(`polski.js: ${licz} wywołań, ${zm} zmian / 10 s`); licz = zm = 0; }, 10000);
    new MutationObserver((zmiany) => {
      licz++; zm += zmiany.length;
      for (const z of zmiany) {
        if (z.type === "childList") z.addedNodes.forEach(wezel);
        else if (z.type === "characterData") tekst(z.target);
        else if (z.type === "attributes") atrybuty(z.target);
      }
    }).observe(document.body, {
      childList: true, subtree: true, characterData: true, attributes: true, attributeFilter: ATRYBUTY,
    });
  };
  document.readyState === "loading" ? document.addEventListener("DOMContentLoaded", start) : start();
})();
