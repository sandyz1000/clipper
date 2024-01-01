use crate::common::icons::{
    IconArrowDown, IconArrowUp, IconAsterisk, IconCamera, IconCircleArrowDown, IconCircleArrowUp,
    IconClock, IconCopy, IconEye, IconEyeSlash, IconGear, IconId, IconInbox, IconPaste,
    IconPenToSquare, IconRectangleList,
};

pub fn paste_as_plain_text_script(what: &str) -> String {
    format!(
        "\nvar text = {}\n\
            copy(text)\n\
            copySelection(text)\n\
            paste()",
        what
    )
}

pub fn global_shortcut_commands() -> Vec<Command> {
    vec![
        create_global_shortcut(
            "Show/hide main window",
            "toggle()",
            IconRectangleList,
            "copyq_global_toggle",
        ),
        create_global_shortcut(
            "Show the tray menu",
            "menu()",
            IconInbox,
            "copyq_global_menu",
        ),
        create_global_shortcut(
            "Show main window under mouse cursor",
            "showAt()",
            IconRectangleList,
            "copyq_global_show_under_mouse",
        ),
        create_global_shortcut(
            "Edit clipboard",
            "edit(-1)",
            IconPenToSquare,
            "copyq_global_edit_clipboard",
        ),
        create_global_shortcut(
            "Edit first item",
            "edit(0)",
            IconPenToSquare,
            "copyq_global_edit_first_item",
        ),
        create_global_shortcut(
            "Copy second item",
            "select(1)",
            IconCopy,
            "copyq_global_copy_second_item",
        ),
        create_global_shortcut(
            "Show action dialog",
            "action()",
            IconGear,
            "copyq_global_show_action_dialog",
        ),
        create_global_shortcut(
            "Create new item",
            "edit()",
            IconAsterisk,
            "copyq_global_create_new_item",
        ),
        create_global_shortcut(
            "Copy next item",
            "next()",
            IconArrowDown,
            "copyq_global_copy_next",
        ),
        create_global_shortcut(
            "Copy previous item",
            "previous()",
            IconArrowUp,
            "copyq_global_copy_previous",
        ),
        create_global_shortcut(
            "Paste clipboard as plain text",
            &paste_as_plain_text_script("clipboard()"),
            IconPaste,
            "copyq_global_paste_clipboard_plain",
        ),
        create_global_shortcut(
            "Disable clipboard storing",
            "disable()",
            IconEyeSlash,
            "copyq_global_disable_clipboard_store",
        ),
        create_global_shortcut(
            "Enable clipboard storing",
            "enable()",
            IconEye,
            "copyq_global_enable_clipboard_store",
        ),
        create_global_shortcut(
            "Paste and copy next",
            "paste(); next()",
            IconCircleArrowDown,
            "copyq_global_paste_copy_next",
        ),
        create_global_shortcut(
            "Paste and copy previous",
            "paste(); previous()",
            IconCircleArrowUp,
            "copyq_global_paste_copy_previous",
        ),
        create_global_shortcut(
            "Take screenshot",
            "screenshotSelect(); copy('image/png', imageData);",
            IconCamera,
            "copyq_global_screenshot",
        ),
        create_global_shortcut(
            "Paste current date and time",
            &command_paste_date_time(),
            IconClock,
            "copyq_global_paste_datetime",
        ),
    ]
}

fn command_paste_date_time() -> String {
    let format = "format";
    format!(
        "\n\
            // http://doc.qt.io/qt-5/qdatetime.html#toString\n\
            var format = '{}'\n\
            var dateTime = dateString(format)\n\
            copy(dateTime)\n\
            copySelection(dateTime)\n\
            paste()",
        format
    )
}

fn create_global_shortcut(name: &str, script: &str, icon: IconId, internal_id: &str) -> Command {
    let mut command = Command::new();
    command.internal_id = internal_id.to_string();
    command.name = name.to_string();
    command.cmd = format!("copyq: {}", script);
    command.icon = Some(icon);
    command.is_global_shortcut = true;
    command
}

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Command {
    name: String,
    re: String,
    wndre: String,
    match_cmd: String,
    cmd: String,
    sep: String,
    input: String,
    output: String,
    wait: bool,
    automatic: bool,
    display: bool,
    in_menu: bool,
    is_global_shortcut: bool,
    is_script: bool,
    transform: bool,
    remove: bool,
    hide_window: bool,
    enable: bool,
    icon: Option<String>,
    shortcuts: Vec<String>,
    global_shortcuts: Vec<String>,
    tab: Option<String>,
    output_tab: Option<String>,
    internal_id: String,
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.re == other.re
            && self.wndre == other.wndre
            && self.match_cmd == other.match_cmd
            && self.cmd == other.cmd
            && self.sep == other.sep
            && self.input == other.input
            && self.output == other.output
            && self.wait == other.wait
            && self.automatic == other.automatic
            && self.display == other.display
            && self.in_menu == other.in_menu
            && self.is_global_shortcut == other.is_global_shortcut
            && self.is_script == other.is_script
            && self.transform == other.transform
            && self.remove == other.remove
            && self.hide_window == other.hide_window
            && self.enable == other.enable
            && self.icon == other.icon
            && self.shortcuts == other.shortcuts
            && self.global_shortcuts == other.global_shortcuts
            && self.tab == other.tab
            && self.output_tab == other.output_tab
            && self.internal_id == other.internal_id
    }
}

impl Command {
    pub fn new(
        name: String,
        re: String,
        wndre: String,
        match_cmd: String,
        cmd: String,
        sep: String,
        input: String,
        output: String,
        wait: bool,
        automatic: bool,
        display: bool,
        in_menu: bool,
        is_global_shortcut: bool,
        is_script: bool,
        transform: bool,
        remove: bool,
        hide_window: bool,
        enable: bool,
        icon: Option<String>,
        shortcuts: Vec<String>,
        global_shortcuts: Vec<String>,
        tab: Option<String>,
        output_tab: Option<String>,
        internal_id: String,
    ) -> Self {
        Command {
            name,
            re,
            wndre,
            match_cmd,
            cmd,
            sep,
            input,
            output,
            wait,
            automatic,
            display,
            in_menu,
            is_global_shortcut,
            is_script,
            transform,
            remove,
            hide_window,
            enable,
            icon,
            shortcuts,
            global_shortcuts,
            tab,
            output_tab,
            internal_id,
        }
    }

    pub fn type_(&self) -> CommandType {
        let mut command_type = 0;
        if self.automatic {
            command_type |= CommandType::Automatic;
        }
        if self.display {
            command_type |= CommandType::Display;
        }
        if self.is_global_shortcut {
            command_type |= CommandType::GlobalShortcut;
        }
        if self.in_menu && !self.name.is_empty() {
            command_type |= CommandType::Menu;
        }
        if self.is_script {
            command_type = CommandType::Script;
        }
        if command_type == CommandType::None {
            command_type = CommandType::Invalid;
        }
        if !self.enable {
            command_type |= CommandType::Disabled;
        }
        command_type
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum CommandType {
    // pub(crate) value: i32,
    None(i32),           // 0;
    Invalid(i32),        // 1;
    Automatic(i32),      // 2;
    Display(i32),        // 4;
    GlobalShortcut(i32), // 8;
    Menu(i32),           // 16;
    Script(i32),         // 32;
    Disabled(i32),       // 64;
}

impl CommandType {
    pub fn new(value: i32) -> Self {
        todo!()
    }
}

impl From<i32> for CommandType {
    fn from(value: i32) -> Self {
        todo!()
    }
}

impl PartialEq<i32> for CommandType {
    fn eq(&self, other: &i32) -> bool {
        self.value == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_equality() {
        let command1 = Command::new(
            "Name".to_string(),
            "re".to_string(),
            "wndre".to_string(),
            "match_cmd".to_string(),
            "cmd".to_string(),
            "sep".to_string(),
            "input".to_string(),
            "output".to_string(),
            true,
            false,
            true,
            false,
            false,
            true,
            false,
            false,
            false,
            true,
            Some("icon".to_string()),
            vec!["shortcut1".to_string()],
            vec!["global_shortcut1".to_string()],
            Some("tab".to_string()),
            Some("output_tab".to_string()),
            "internal_id".to_string(),
        );

        let command2 = Command::new(
            "Name".to_string(),
            "re".to_string(),
            "wndre".to_string(),
            "match_cmd".to_string(),
            "cmd".to_string(),
            "sep".to_string(),
            "input".to_string(),
            "output".to_string(),
            true,
            false,
            true,
            false,
            false,
            true,
            false,
            false,
            false,
            true,
            Some("icon".to_string()),
            vec!["shortcut1".to_string()],
            vec!["global_shortcut1".to_string()],
            Some("tab".to_string()),
            Some("output_tab".to_string()),
            "internal_id".to_string(),
        );

        assert_eq!(command1, command2);
    }

    #[test]
    fn test_command_type_from_i32() {
        let command_type = CommandType::from(8);
        assert_eq!(command_type, CommandType::GlobalShortcut(8));
    }

    #[test]
    fn test_command_type_equality_i32() {
        let command_type = CommandType::GlobalShortcut(8);
        assert_eq!(command_type, 8);
    }
}
