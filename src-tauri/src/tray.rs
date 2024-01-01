
use tauri::{
    Manager, 
    AppHandle, 
    SystemTray, 
    SystemTrayMenuItem, 
    SystemTrayMenu, 
    CustomMenuItem, 
    SystemTrayEvent, 
};

/// Set the UI on the system tray 
pub fn main_menu() -> SystemTray {
    let preference: CustomMenuItem =  CustomMenuItem::new("settings", "Preferences");
    let about: CustomMenuItem  = CustomMenuItem::new("about ".to_string(), "About");
    let quit: CustomMenuItem = CustomMenuItem::new("quit ".to_string(), "Quit");

    let tray_menu = SystemTrayMenu::new()
    .add_native_item(SystemTrayMenuItem::Separator)
    .add_item(preference)
    .add_native_item(SystemTrayMenuItem::Separator)
    .add_item(about)
    .add_item(quit);

    SystemTray::new().with_menu(tray_menu)

}


pub fn handler(app: &AppHandle, event: SystemTrayEvent) {
    let window = app.get_window("main").unwrap();

    if let SystemTrayEvent::MenuItemClick { id, .. } = event {
        match id.as_str() {
            "about" => app.windows().values().for_each(|window| {
                // Show about window
                window.show().unwrap();
                window.set_focus().unwrap();
            }),
            "quit" => std::process::exit(0),
            "settings" => {
                // open settings window
                window.show().unwrap();
                window.set_focus().unwrap();
            },
            // TODO: Copy the content in the clipboard
            _ => { }
        }
    }
}