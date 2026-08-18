# TextForge

> Persönliches Text-Transformations-Tool — Tauri 2.x · SvelteKit · SQLite

## Systemvoraussetzungen (Linux)

### Build-Abhängigkeiten

Zum Kompilieren der Tauri-App werden folgende Systembibliotheken benötigt:

```bash
# Tauri 2.x Build-Abhängigkeiten (Debian/Ubuntu)
sudo apt install pkg-config libsoup-3.0-dev libwebkit2gtk-4.1-dev libgtk-3-dev

# Rust Toolchain (falls noch nicht vorhanden)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Laufzeit-Abhängigkeiten (optional, empfohlen)

```bash
# Clipboard-Monitoring (event-basiert via Wayland — empfohlen)
# Ohne wl-clipboard fällt die App auf Polling (500ms) zurück.
sudo apt install wl-clipboard

# Quell-App-Erkennung (KDE Plasma 6)
# Ohne qdbus6 wird die Quell-App als "Unbekannt" angezeigt.
sudo apt install kde-cli-tools
```

## Entwicklung

Abhängigkeiten installieren und Development-Server starten:

```bash
npm install
npm run dev
```

Tauri-App im Entwicklungsmodus starten:

```bash
cargo tauri dev
```

## Testen

```bash
# Frontend Domain-Core-Tests
npx vitest run

# Rust-Tests
cargo test -p textforge
```

## Produktion

```bash
# Produktions-Build (erstellt AppImage/deb)
cargo tauri build
```

## Datenbank

Die SQLite-Datenbank wird automatisch im plattform-spezifischen App-Data-Verzeichnis erstellt:

- **Linux**: `~/.local/share/com.textforge.dev/textforge.db`

Falls eine alte `app.db` im Projektverzeichnis existiert (vor diesem Update), wird sie automatisch einmalig migriert.

## Architektur

Siehe [CLAUDE.md](./CLAUDE.md) für die vollständige Architektur-Dokumentation und Arbeitsanweisungen.
