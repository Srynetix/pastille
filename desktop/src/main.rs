// SPDX-License-Identifier: MIT OR Apache-2.0

//! The desktop shell: the web app, in a window of its own (C-68, ADR 0013).
//!
//! Nothing here simulates anything. The window shows the same `app/dist` the
//! browser is served, with the same wasm core inside it, so there is one app
//! and two ways to open it. What the shell adds is the one thing the web view
//! cannot do the way a browser does: put a file on disk. A browser saves a
//! circuit by following a download link, and the system web view on macOS
//! ignores those, so saving — and opening, to match — goes through the
//! platform's own dialogs instead. Links are the other thing: a page link with
//! `target="_blank"` asks for a new window, which the web view never opens, so
//! a datasheet goes to the system's browser.

// A release build is a windowed app on Windows: no console behind it.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use tauri::AppHandle;
use tauri::webview::PageLoadEvent;
use tauri_plugin_dialog::{DialogExt, FilePath};
use tauri_plugin_opener::OpenerExt;

/// Only circuits are offered, the way the browser's picker offers them.
const FILTER: (&str, &[&str]) = ("Circuit", &["json"]);

/// Asks where to save `text`, and writes it there.
///
/// `false` when the dialog was dismissed: not an error, and the app says
/// nothing, the way it says nothing when a browser's download is cancelled.
///
/// A circuit unless the app says otherwise: a parts list (B-11) is saved as
/// what it is, `kind` and `extension` naming it, because a dialog offering
/// only circuits renames `parts-list.csv` to a `.json`.
#[tauri::command]
async fn save_circuit(
    app: AppHandle,
    text: String,
    name: String,
    kind: Option<String>,
    extension: Option<String>,
) -> Result<bool, String> {
    let (kind, extension) = match (kind.as_deref(), extension.as_deref()) {
        (Some(kind), Some(extension)) => (kind, extension),
        _ => (FILTER.0, FILTER.1[0]),
    };
    let chosen = app
        .dialog()
        .file()
        .add_filter(kind, &[extension])
        .set_file_name(&name)
        .blocking_save_file();
    let Some(path) = chosen.map(local) else {
        return Ok(false);
    };
    let path = path?;
    std::fs::write(&path, text)
        .map_err(|error| format!("could not save {}: {error}", path.display()))?;
    Ok(true)
}

/// Asks for a circuit to open, and reads it, or nothing if the dialog was
/// dismissed.
#[tauri::command]
async fn open_circuit(app: AppHandle) -> Result<Option<String>, String> {
    let chosen = app
        .dialog()
        .file()
        .add_filter(FILTER.0, FILTER.1)
        .blocking_pick_file();
    let Some(path) = chosen.map(local) else {
        return Ok(None);
    };
    let path = path?;
    std::fs::read_to_string(&path)
        .map(Some)
        .map_err(|error| format!("could not open {}: {error}", path.display()))
}

/// Asks for a program to load into an Uno — the Intel HEX the Arduino IDE
/// exports (A-03) — and reads it: its file name and its text, or nothing if
/// the dialog was dismissed.
#[tauri::command]
async fn open_program(app: AppHandle) -> Result<Option<(String, String)>, String> {
    let chosen = app
        .dialog()
        .file()
        .add_filter("Arduino program", &["hex"])
        .blocking_pick_file();
    let Some(path) = chosen.map(local) else {
        return Ok(None);
    };
    let path = path?;
    let name = path.file_name().map_or_else(
        || "program.hex".to_string(),
        |name| name.to_string_lossy().into_owned(),
    );
    std::fs::read_to_string(&path)
        .map(|text| Some((name, text)))
        .map_err(|error| format!("could not open {}: {error}", path.display()))
}

/// A dialog's answer as a path on this machine. Only mobile platforms answer
/// with anything else, and this shell is not built for them.
fn local(chosen: FilePath) -> Result<PathBuf, String> {
    chosen
        .into_path()
        .map_err(|error| format!("not a file on this machine: {error}"))
}

/// Opens `url` in the system's browser.
///
/// Only `https:` is followed. The links the app offers are datasheets on
/// manufacturers' sites; anything else reaching here — a `file:` path, a
/// custom scheme that starts another app — is not something a circuit should
/// be able to ask for, so it is refused rather than handed to the system.
#[tauri::command]
fn open_link(app: AppHandle, url: String) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err(format!("not a web address: {url}"));
    }
    app.opener()
        .open_url(&url, None::<&str>)
        .map_err(|error| format!("could not open {url}: {error}"))
}

/// What `just desktop-smoke` runs inside the window once the page has loaded.
///
/// Nothing on this machine can look at a native window for us, so the window
/// looks at itself: it waits for the core to say it is ready — which is the
/// wasm bundle compiled and running under this shell's content security
/// policy, in the system's web view rather than Chrome — opens the first
/// example if the canvas is empty, and waits for a second of it to be
/// simulated. Then it checks that a link the shell must not follow is refused,
/// which is `open_link` registered and its opener running without a browser
/// being opened. Then it reports what the status bar says, and the shell exits
/// with it.
const SMOKE: &str = r#"
(() => {
    const started = Date.now()
    const invoke = (command, args) => window.__TAURI_INTERNALS__.invoke(command, args)
    const report = (ok, detail) => invoke('smoke_report', { ok, detail })
    const refusesFile = () =>
        invoke('open_link', { url: 'file:///etc/hosts' }).then(
            () => false,
            (error) => String(error).startsWith('not a web address'),
        )
    const tick = () => {
        const text = document.body.innerText
        const example = [...document.querySelectorAll('button')]
            .find((button) => button.textContent.trim().startsWith('A lit LED'))
        if (example) example.click()
        const simulated = /([0-9]+\.[0-9]) s simulated/.exec(text)
        if (simulated && Number(simulated[1]) >= 1) {
            return refusesFile().then((refused) =>
                refused
                    ? report(true, simulated[0])
                    : report(false, 'open_link did not refuse a file: address'),
            )
        }
        if (Date.now() - started > 20000) {
            return report(false, text.slice(-400))
        }
        setTimeout(tick, 200)
    }
    tick()
})()
"#;

/// Where the smoke test's window reports back.
#[tauri::command]
fn smoke_report(app: AppHandle, ok: bool, detail: String) {
    println!(
        "{} {detail}",
        if ok {
            "smoke: ok —"
        } else {
            "smoke: FAILED —"
        }
    );
    app.exit(if ok { 0 } else { 1 });
}

fn main() {
    let smoke = std::env::var_os("PASTILLE_SMOKE").is_some();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            save_circuit,
            open_circuit,
            open_program,
            open_link,
            smoke_report
        ])
        .on_page_load(move |webview, payload| {
            if smoke && payload.event() == PageLoadEvent::Finished {
                let _ = webview.eval(SMOKE);
            }
        })
        .run(tauri::generate_context!())
        .expect("the desktop shell could not start");
}
