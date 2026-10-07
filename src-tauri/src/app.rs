use crate::{
    clipboard::{self, Selection},
    config::{self, Config},
    detection::{self, Analysis},
    hotkeys::Hotkeys,
    layout::{self, Conversion, Engine, Layout},
};
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Duration,
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_notification::NotificationExt;
use windows::{
    core::w,
    Win32::{
        Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, HWND},
        System::Threading::CreateMutexW,
        UI::WindowsAndMessaging::{
            FindWindowExW, GetForegroundWindow, PostMessageW, SetForegroundWindow, HWND_MESSAGE,
        },
    },
};

struct Pending {
    selection: Selection,
    analysis: Analysis,
}
type WindowBounds = (tauri::PhysicalSize<u32>, tauri::PhysicalPosition<i32>, bool);
pub struct AppState {
    config: Mutex<Config>,
    engine: Mutex<Engine>,
    hotkeys: Mutex<Option<Hotkeys>>,
    pending: Mutex<Option<Pending>>,
    busy: AtomicBool,
    paused: AtomicBool,
    compact_bounds: Mutex<Option<WindowBounds>>,
    path: PathBuf,
    errors: Mutex<Vec<String>>,
}
struct Instance(usize);
impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(HANDLE(self.0 as *mut _));
        }
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Bootstrap {
    config: Config,
    layouts: Vec<Layout>,
    errors: Vec<String>,
    paused: bool,
    version: &'static str,
    system_language: String,
}
fn lock<'a, T>(m: &'a Mutex<T>) -> Result<std::sync::MutexGuard<'a, T>, String> {
    m.lock().map_err(|_| "state_unavailable".into())
}

#[tauri::command]
fn bootstrap(state: State<AppState>) -> Result<Bootstrap, String> {
    // SAFETY: query-only user UI language API.
    let lang = unsafe { windows::Win32::Globalization::GetUserDefaultUILanguage() };
    Ok(Bootstrap {
        config: lock(&state.config)?.clone(),
        layouts: layout::available(),
        errors: lock(&state.errors)?.clone(),
        paused: state.paused.load(Ordering::Relaxed),
        version: env!("CARGO_PKG_VERSION"),
        system_language: if lang & 0x3ff == 0x29 { "fa" } else { "en" }.into(),
    })
}
#[tauri::command]
fn refresh_layouts(state: State<AppState>) -> Result<Vec<Layout>, String> {
    lock(&state.engine)?.clear();
    Ok(layout::available())
}
#[tauri::command]
async fn convert_text(
    app: AppHandle,
    text: String,
    source: String,
    target: String,
) -> Result<Conversion, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let result = lock(&state.engine)?.convert(&text, &source, &target);
        result
    })
    .await
    .map_err(|_| "conversion_failed")?
}
#[tauri::command]
async fn analyze_text(app: AppHandle, text: String) -> Result<Analysis, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let s = app.state::<AppState>();
        let c = lock(&s.config)?.clone();
        let result = detection::analyze(
            &mut *lock(&s.engine)?,
            &text,
            &layout::detection_layouts(&c),
            &c,
            &layout::foreground_layout(),
        );
        result
    })
    .await
    .map_err(|_| "conversion_failed")?
}
#[tauri::command]
async fn save_settings(app: AppHandle, config: Config) -> Result<Config, String> {
    let worker = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        config.validate()?;
        let s = worker.state::<AppState>();
        let mut old = lock(&s.config)?;
        let layouts = layout::available();
        if [&config.source, &config.target]
            .iter()
            .any(|id| !id.is_empty() && !layouts.iter().any(|l| &l.id == *id))
            || config
                .preferred_layouts
                .iter()
                .any(|id| !layouts.iter().any(|l| &l.id == id))
        {
            return Err("layout_unavailable".into());
        }
        let hotkeys = lock(&s.hotkeys)?;
        let keys = hotkeys.as_ref().ok_or("hotkey_init_failed")?;
        keys.update(config.clone())?;
        if let Err(e) = config::startup(config.launch_at_startup).and_then(|_| config.save(&s.path))
        {
            let _ = keys.update(old.clone());
            let _ = config::startup(old.launch_at_startup);
            return Err(e);
        }
        *old = config.clone();
        lock(&s.errors)?.retain(|e| e != "hotkey_collision");
        Ok(config)
    })
    .await
    .map_err(|_| "settings_write_failed")?;
    if result.is_ok() {
        let _ = refresh_tray(&app);
        let _ = app.emit("settings-changed", ());
    }
    result
}
#[tauri::command]
fn read_clipboard() -> Result<String, String> {
    clipboard::read_text()
}
#[tauri::command]
fn write_clipboard(text: String) -> Result<(), String> {
    clipboard::write_text(&text)
}
#[tauri::command]
fn set_paused(app: AppHandle, paused: bool) -> Result<(), String> {
    app.state::<AppState>()
        .paused
        .store(paused, Ordering::Relaxed);
    refresh_tray(&app)?;
    Ok(())
}
#[tauri::command]
fn pending_selection(state: State<AppState>) -> Result<Option<Analysis>, String> {
    let mut p = lock(&state.pending)?;
    if p.as_ref()
        .is_some_and(|p| p.selection.created.elapsed() > Duration::from_secs(120))
    {
        *p = None;
    }
    Ok(p.as_ref().map(|p| p.analysis.clone()))
}
#[tauri::command]
fn cancel_selection(state: State<AppState>) -> Result<(), String> {
    *lock(&state.pending)? = None;
    Ok(())
}
#[tauri::command]
async fn apply_selection(app: AppHandle, index: usize) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let s = app.state::<AppState>();
        if s.busy.swap(true, Ordering::AcqRel) {
            return Err("busy".into());
        }
        let _busy = Busy(&s.busy);
        let p = lock(&s.pending)?.take().ok_or("selection_expired")?;
        let c = p
            .analysis
            .candidates
            .get(index)
            .ok_or("invalid_candidate")?;
        let restore = lock(&s.config)?.restore_clipboard;
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.hide();
        }
        // SAFETY: foreground activation is user initiated from the candidate picker; verified before injection.
        unsafe {
            let _ = SetForegroundWindow(HWND(p.selection.hwnd as *mut _));
        }
        std::thread::sleep(Duration::from_millis(100));
        let result = clipboard::replace(&p.selection, &c.text, restore);
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show();
        }
        result
    })
    .await
    .map_err(|_| "conversion_failed")?
}

#[tauri::command]
fn window_action(window: tauri::WebviewWindow, action: String) -> Result<(), String> {
    let result = match action.as_str() {
        "minimize" => window.minimize(),
        "maximize" => {
            if window.is_maximized().map_err(|_| "window_failed")? {
                window.unmaximize()
            } else {
                window.maximize()
            }
        }
        "close" => window.close(),
        "drag" => window.start_dragging(),
        _ => return Err("window_failed".into()),
    };
    result.map_err(|_| "window_failed".into())
}
#[tauri::command]
fn set_compact(
    window: tauri::WebviewWindow,
    state: State<AppState>,
    compact: bool,
) -> Result<(), String> {
    let mut bounds = lock(&state.compact_bounds)?;
    let change = || -> tauri::Result<()> {
        if compact {
            if bounds.is_some() {
                return Ok(());
            }
            let maximized = window.is_maximized()?;
            if maximized {
                window.unmaximize()?;
            }
            let previous = (window.inner_size()?, window.outer_position()?, maximized);
            window.set_min_size(Some(tauri::LogicalSize::new(520.0, 340.0)))?;
            window.set_size(tauri::LogicalSize::new(800.0, 440.0))?;
            *bounds = Some(previous);
        } else if let Some((size, position, maximized)) = *bounds {
            window.set_min_size(Some(tauri::LogicalSize::new(600.0, 500.0)))?;
            window.set_size(size)?;
            window.set_position(position)?;
            if maximized {
                window.maximize()?;
            }
            *bounds = None;
        }
        Ok(())
    };
    let mut change = change;
    change().map_err(|_| "window_failed".into())
}

struct Busy<'a>(&'a AtomicBool);
impl Drop for Busy<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
fn notify(app: &AppHandle, code: &str) {
    let s = app.state::<AppState>();
    let Ok(c) = lock(&s.config) else {
        return;
    };
    if c.notifications {
        let body = if c.ui_language == "fa" {
            match code {
                "converted" => "متن با موفقیت اصلاح شد.",
                "unchanged" => "متن بدون تغییر باقی ماند.",
                _ => "اصلاح متن انجام نشد. برای جزئیات PIMXSWAP را باز کنید.",
            }
        } else {
            match code {
                "converted" => "Selected text corrected.",
                "unchanged" => "Text left unchanged.",
                _ => "Correction could not be applied. Open PIMXSWAP for details.",
            }
        };
        let _ = app
            .notification()
            .builder()
            .title("PIMXSWAP")
            .body(body)
            .show();
    }
    let _ = app.emit("operation-status", code);
}
fn shortcut(app: AppHandle, id: u32) {
    if id == 3 {
        let _ = open_window(&app, "converter");
        return;
    }
    let s = app.state::<AppState>();
    if s.paused.load(Ordering::Relaxed) || s.busy.swap(true, Ordering::AcqRel) {
        return;
    }
    let hwnd = unsafe { GetForegroundWindow().0 as usize };
    let current = layout::foreground_layout();
    std::thread::spawn(move || {
        let s = app.state::<AppState>();
        let _busy = Busy(&s.busy);
        let result = (|| -> Result<(), String> {
            let c = lock(&s.config)?.clone();
            let selection = clipboard::capture(hwnd, c.restore_clipboard)?;
            if id == 2 {
                if c.source.is_empty() || c.target.is_empty() {
                    return Err("choose_layouts".into());
                }
                // Swap uses the configured pair. Script coverage decides which side the selection came from.
                let mut e = lock(&s.engine)?;
                let a = e.convert(&selection.text, &c.source, &c.target)?;
                let b = e.convert(&selection.text, &c.target, &c.source)?;
                let output = if b.mapped > a.mapped { b } else { a };
                drop(e);
                if output.text == selection.text {
                    notify(&app, "unchanged");
                    return Ok(());
                }
                clipboard::replace(&selection, &output.text, c.restore_clipboard)?;
                notify(&app, "converted");
                return Ok(());
            }
            let analysis = detection::analyze(
                &mut *lock(&s.engine)?,
                &selection.text,
                &layout::detection_layouts(&c),
                &c,
                &current,
            )?;
            match analysis.action.as_str() {
                "apply" => {
                    let best = analysis.candidates.first().ok_or("invalid_candidate")?;
                    clipboard::replace(&selection, &best.text, c.restore_clipboard)?;
                    notify(&app, "converted");
                }
                "choose" => {
                    *lock(&s.pending)? = Some(Pending {
                        selection,
                        analysis,
                    });
                    open_window(&app, "converter")?;
                    let _ = app.emit("selection-ready", ());
                }
                _ => notify(&app, "unchanged"),
            }
            Ok(())
        })();
        if let Err(e) = result {
            if let Ok(mut errors) = lock(&s.errors) {
                errors.clear();
                errors.push(e.clone());
            }
            notify(&app, &e);
        }
    });
}
fn open_window(app: &AppHandle, page: &str) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|_| "window_failed")?;
        window.unminimize().map_err(|_| "window_failed")?;
        window.set_focus().map_err(|_| "window_failed")?;
        let _ = window.emit("navigate", page);
        return Ok(());
    }
    WebviewWindowBuilder::new(
        app,
        "main",
        WebviewUrl::App(format!("index.html?page={page}").into()),
    )
    .title("PIMXSWAP")
    .inner_size(1060.0, 790.0)
    .min_inner_size(600.0, 500.0)
    .decorations(false)
    .center()
    .build()
    .map_err(|_| "window_failed")?;
    Ok(())
}
fn menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let s = app.state::<AppState>();
    let fa = s
        .config
        .lock()
        .map(|c| c.ui_language == "fa")
        .unwrap_or(false);
    let paused = s.paused.load(Ordering::Relaxed);
    let labels = if fa {
        [
            "باز کردن PIMXSWAP",
            "اصلاح متن انتخاب‌شده",
            "تنظیمات",
            "تغییر زبان",
            "توقف میانبرها",
            "ادامهٔ میانبرها",
            "خروج",
        ]
    } else {
        [
            "Open PIMXSWAP",
            "Auto Fix selected text",
            "Settings",
            "Switch language",
            "Pause shortcuts",
            "Resume shortcuts",
            "Exit",
        ]
    };
    let layouts = layout::installed();
    let active = layout::foreground_layout();
    let name = layouts
        .iter()
        .find(|l| l.id == active)
        .map_or("—", |l| l.name.as_str());
    Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "open", labels[0], true, None::<&str>)?,
            &MenuItem::with_id(app, "fix", labels[1], true, None::<&str>)?,
            &MenuItem::with_id(
                app,
                "layout",
                format!("{}: {name}", if fa { "چیدمان" } else { "Layout" }),
                false,
                None::<&str>,
            )?,
            &MenuItem::with_id(app, "settings", labels[2], true, None::<&str>)?,
            &MenuItem::with_id(app, "language", labels[3], true, None::<&str>)?,
            &MenuItem::with_id(
                app,
                "pause",
                labels[if paused { 5 } else { 4 }],
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(app, "exit", labels[6], true, None::<&str>)?,
        ],
    )
}
fn refresh_tray(app: &AppHandle) -> Result<(), String> {
    if let Some(tray) = app.tray_by_id("pimxswap") {
        tray.set_menu(Some(menu(app).map_err(|_| "tray_failed")?))
            .map_err(|_| "tray_failed")?;
    }
    Ok(())
}
pub fn run() {
    let instance = unsafe { CreateMutexW(None, false, w!("Local\\PIMXSWAP.Desktop")) };
    let Ok(handle) = instance else {
        return;
    };
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        // SAFETY: locate only our named message-only instance window and request its editor.
        for _ in 0..20 {
            if let Ok(window) = unsafe {
                FindWindowExW(
                    Some(HWND_MESSAGE),
                    None,
                    w!("STATIC"),
                    w!("PIMXSWAP.Instance"),
                )
            } {
                unsafe {
                    let _ = PostMessageW(
                        Some(window),
                        crate::hotkeys::OPEN_INSTANCE,
                        Default::default(),
                        Default::default(),
                    );
                }
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        unsafe {
            let _ = CloseHandle(handle);
        }
        return;
    }
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(Instance(handle.0 as usize))
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            refresh_layouts,
            convert_text,
            analyze_text,
            save_settings,
            read_clipboard,
            write_clipboard,
            set_paused,
            pending_selection,
            cancel_selection,
            apply_selection,
            window_action,
            set_compact
        ])
        .setup(|app| {
            let path = app.path().app_config_dir()?.join("settings.json");
            let mut errors = vec![];
            let mut config = Config::load(&path).unwrap_or_else(|e| {
                errors.push(e);
                Config::default()
            });
            let layouts = layout::installed();
            if !layout::available().iter().any(|l| l.id == config.source) {
                config.source = layouts
                    .iter()
                    .find(|l| l.language == "en")
                    .or(layouts.first())
                    .map_or(String::new(), |l| l.id.clone());
            }
            if !layout::available().iter().any(|l| l.id == config.target) {
                config.target = layouts
                    .iter()
                    .find(|l| l.language == "fa")
                    .or_else(|| layouts.iter().find(|l| l.id != config.source))
                    .map_or(config.source.clone(), |l| l.id.clone());
            }
            let catalog = layout::available();
            config
                .preferred_layouts
                .retain(|id| catalog.iter().any(|l| &l.id == id));
            let background =
                std::env::args().any(|a| a == "--background") || config.start_minimized;
            let h = app.handle().clone();
            let (hotkeys, error) =
                Hotkeys::start(config.clone(), move |id| shortcut(h.clone(), id))?;
            if let Some(e) = error {
                errors.push(e);
            }
            app.manage(AppState {
                config: Mutex::new(config),
                engine: Mutex::new(Engine::default()),
                hotkeys: Mutex::new(Some(hotkeys)),
                pending: Mutex::new(None),
                busy: AtomicBool::new(false),
                paused: AtomicBool::new(false),
                compact_bounds: Mutex::new(None),
                path,
                errors: Mutex::new(errors),
            });
            TrayIconBuilder::with_id("pimxswap")
                .icon(tauri::image::Image::from_bytes(include_bytes!(
                    "../icons/tray.png"
                ))?)
                .tooltip("PIMXSWAP")
                .menu(&menu(app.handle())?)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if matches!(
                        event,
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        }
                    ) {
                        let _ = open_window(tray.app_handle(), "converter");
                    }
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        let _ = open_window(app, "converter");
                    }
                    "settings" => {
                        let _ = open_window(app, "settings");
                    }
                    "fix" => shortcut(app.clone(), 1),
                    "pause" => {
                        let s = app.state::<AppState>();
                        s.paused.fetch_xor(true, Ordering::Relaxed);
                        let _ = refresh_tray(app);
                        let _ = app.emit("settings-changed", ());
                    }
                    "language" => {
                        let s = app.state::<AppState>();
                        if let Ok(mut c) = s.config.lock() {
                            let mut new = c.clone();
                            new.ui_language =
                                if c.ui_language == "fa" { "en" } else { "fa" }.into();
                            if new.save(&s.path).is_ok() {
                                *c = new;
                            }
                        }
                        let _ = refresh_tray(app);
                        let _ = app.emit("settings-changed", ());
                    }
                    "exit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            if !background {
                open_window(app.handle(), "converter")?;
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let s = app.state::<AppState>();
                if s.config.lock().map(|c| c.minimize_to_tray).unwrap_or(true) {
                    api.prevent_close();
                    if let Ok(mut bounds) = s.compact_bounds.lock() {
                        *bounds = None;
                    }
                    if let Some(w) = app.get_webview_window(window.label()) {
                        let _ = w.destroy();
                    }
                    if let Ok(mut p) = s.pending.lock() {
                        *p = None;
                    }
                } else {
                    app.exit(0);
                }
            } else if matches!(event, tauri::WindowEvent::Focused(true)) {
                let _ = refresh_tray(window.app_handle());
            }
        });
    match builder.build(tauri::generate_context!()) {
        Ok(app) => app.run(|_, event| {
            if let tauri::RunEvent::ExitRequested {
                code: None, api, ..
            } = event
            {
                // Destroying the editor's last WebView must leave tray and hotkeys running.
                api.prevent_exit();
            }
        }),
        Err(error) => eprintln!("PIMXSWAP startup failed: {error}"),
    }
}
