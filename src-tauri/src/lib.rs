use tauri::{
    AppHandle, Manager, WebviewWindow,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

// ═══════════════════════════════════════════════════════════
//  系统托盘 + 右键菜单
// ═══════════════════════════════════════════════════════════

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let show_i = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let hide_i = MenuItem::with_id(app, "hide", "隐藏窗口", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "退出 Ybooks", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_i, &hide_i, &quit_i])?;

    TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Ybooks 工作看板")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.unminimize();
                    let _ = w.set_focus();
                }
            }
            "hide" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    if w.is_visible().unwrap_or(false) {
                        let _ = w.hide();
                    } else {
                        let _ = w.show();
                        let _ = w.unminimize();
                        let _ = w.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

// ═══════════════════════════════════════════════════════════
//  关闭按钮 → 最小化到托盘
// ═══════════════════════════════════════════════════════════

fn prevent_close(window: &WebviewWindow) {
    let w = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = w.hide();
        }
    });
}

// ═══════════════════════════════════════════════════════════
//  Tauri 命令（前端 JS 调用）
// ═══════════════════════════════════════════════════════════

#[tauri::command]
fn send_notification(app: AppHandle, title: String, body: String) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app.notification()
        .builder()
        .title(&title)
        .body(&body)
        .show();
}

#[tauri::command]
fn minimize_window(window: WebviewWindow) {
    let _ = window.minimize();
}

#[tauri::command]
fn toggle_maximize(window: WebviewWindow) {
    if window.is_maximized().unwrap_or(false) {
        let _ = window.unmaximize();
    } else {
        let _ = window.maximize();
    }
}

#[tauri::command]
fn hide_window(window: WebviewWindow) {
    let _ = window.hide();
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn is_autostart_enabled(app: AppHandle) -> bool {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[tauri::command]
fn set_autostart(app: AppHandle, enabled: bool) -> bool {
    use tauri_plugin_autostart::ManagerExt;
    let mgr = app.autolaunch();
    let result = if enabled {
        mgr.enable()
    } else {
        mgr.disable()
    };
    result.is_ok()
}

// ═══════════════════════════════════════════════════════════
//  应用入口
// ═══════════════════════════════════════════════════════════

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    // ── 单实例锁（仅桌面端） ──
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }));
    }

    builder
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart".to_string()]),
        ))
        // ── 全局快捷键 ──
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) && !w.is_minimized().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.unminimize();
                                let _ = w.set_focus();
                            }
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            // 系统托盘
            build_tray(app.handle())?;

            // 关闭→最小化到托盘
            if let Some(window) = app.get_webview_window("main") {
                prevent_close(&window);
            }

            // 注册全局快捷键 Ctrl+Shift+Y
            let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyY);
            app.global_shortcut().register(shortcut)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            send_notification,
            minimize_window,
            toggle_maximize,
            hide_window,
            quit_app,
            is_autostart_enabled,
            set_autostart,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Ybooks Desktop");
}
