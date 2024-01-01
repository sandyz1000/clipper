// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tauri::Manager;
mod common;
mod tray;


// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
    .setup(|app| {
        
        let window = app.get_window("main").unwrap();

        window.set_minimizable(false);
        window.set_resizable(false);
        Ok(())
    })
    .on_window_event(|event: tauri::GlobalWindowEvent| match event.event() {
        tauri::WindowEvent::CloseRequested { api, .. } => {
          event.window().hide().unwrap();
          api.prevent_close();
        }
        _ => {}
    })
    
    // This will add menu on system tray
    .system_tray(tray::main_menu())  
    
    // Adding event handler on systemtray menu
    .on_system_tray_event(tray::handler)
    
    // Invoke a command
    .invoke_handler(tauri::generate_handler![greet])
    // Build the app
    .build(tauri::generate_context!())
    .expect("error while building tauri application")
    
    // This will prevent the app from exiting
    .run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
        api.prevent_exit();
        }
        _ => {}
    });
}
