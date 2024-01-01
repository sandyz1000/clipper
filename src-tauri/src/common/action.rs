use std::env;
use std::process::{exit, Command, Stdio};
use std::collections::{HashMap, VecDeque};
use std::io::{self, Write};


const MIME_TEXT: &'static str = "text/plain";
const MIME_TEXT_UTF8: &'static str = "text/plain;charset=utf-8";
const MIME_HTML: &'static str = "text/html";
const MIME_URI_LIST: &'static str = "text/uri-list";
const MIME_WINDOW_TITLE: &'static str = "COPYQ_MIME_PREFIX owner-window-title";
const MIME_ITEMS: &'static str = "COPYQ_MIME_PREFIX item";
const MIME_ITEM_NOTES: &'static str = "COPYQ_MIME_PREFIX item-notes";
const MIME_ICON: &'static str = "COPYQ_MIME_PREFIX item-icon";
const MIME_OWNER: &'static str = "COPYQ_MIME_PREFIX owner";
const MIME_CLIPBOARD_MODE: &'static str = "COPYQ_MIME_PREFIX clipboard-mode";
const MIME_CURRENT_TAB: &'static str = "COPYQ_MIME_PREFIX current-tab";
const MIME_SELECTED_ITEMS: &'static str = "COPYQ_MIME_PREFIX selected-items";
const MIME_CURRENT_ITEM: &'static str = "COPYQ_MIME_PREFIX current-item";
const MIME_HIDDEN: &'static str = "COPYQ_MIME_PREFIX hidden";
const MIME_SHORTCUT: &'static str = "COPYQ_MIME_PREFIX shortcut";
const MIME_COLOR: &'static str = "COPYQ_MIME_PREFIX color";
const MIME_OUTPUT_TAB: &'static str = "COPYQ_MIME_PREFIX output-tab";
const MIME_DISPLAY_ITEM_IN_MENU: &'static str = "COPYQ_MIME_PREFIX display-item-in-menu";


pub fn start_process(args: Vec<&str>, mode: std::process::Command) -> io::Result<()> {
    let executable = args[0];

    // Replace "copyq" command with full application path.
    if executable == "copyq" {
        let app_path = env::current_exe()?;
        mode.arg(app_path);
    }

    let process = mode.args(&args[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    Ok(())
}

fn append_and_clear_non_empty<T: Clone>(entry: &mut Vec<T>, container: &mut VecDeque<Vec<T>>) {
    if !entry.is_empty() {
        container.push_back(entry.clone());
        entry.clear();
    }
}

fn get_script_from_label(label: &str, cmd: &str, i: usize, script: &mut String) -> bool {
    let label_str = label.to_string();
    let mid = if i + label.len() <= cmd.len() {
        &cmd[i..i + label.len()]
    } else {
        ""
    };

    if mid == label_str {
        *script = cmd[i + label.len()..].to_string();
        return true;
    }

    false
}

fn parse_commands(cmd: &str, captured_texts: &[String]) -> Vec<Vec<Vec<String>>> {
    let mut lines = Vec::new();
    let mut commands = Vec::new();
    let mut command = Vec::new();
    let mut script = String::new();

    let mut arg = String::new();
    let mut quote: Option<char> = None;
    let mut escape = false;
    let mut percent = false;

    // let re_unescaped_windows_path = regex::Regex::new(r"^\s*['"]?[a-zA-Z]:\\[^\\]").unwrap();
    let allow_escape = !re_unescaped_windows_path.is_match(cmd);

    for (i, c) in cmd.chars().enumerate() {
        if percent {
            if c == '1' || (c >= '2' && c <= '9' && captured_texts.len() > 1) {
                arg.pop();
                arg.push_str(&captured_texts[c.to_digit(10).unwrap() as usize - 1]);
                continue;
            }
        }
        percent = !escape && c == '%';

        if escape {
            escape = false;
            match c {
                'n' => arg.push('\n'),
                't' => arg.push('\t'),
                '\n' => {} // Ignore escaped new line character.
                _ => arg.push(c),
            }
        } else if allow_escape && c == '\\' {
            escape = true;
        } else if let Some(q) = quote {
            if q == c {
                quote = None;
                command.push(arg.clone());
                arg.clear();
            } else {
                arg.push(c);
            }
        } else if c == '\'' || c == '"' {
            quote = Some(c);
        } else if c == '|' {
            append_and_clear_non_empty(&mut arg, &mut command);
            append_and_clear_non_empty(&mut command, &mut commands);
        } else if c == '\n' || c == ';' {
            append_and_clear_non_empty(&mut arg, &mut command);
            append_and_clear_non_empty(&mut command, &mut commands);
            append_and_clear_non_empty(&mut commands, &mut lines);
        } else if c.is_whitespace() {
            if !arg.is_empty() {
                command.push(arg.clone());
                arg.clear();
            }
        } else if c == ':' && i + 1 < cmd.len() && &cmd[i + 1..i + 2] == "\n" {
            // If there is an unescaped colon at the end of a line,
            // treat the rest of the command as a single argument.
            append_and_clear_non_empty(&mut arg, &mut command);
            arg = cmd[i + 2..].to_string();
            break;
        } else {
            if arg.is_empty() && command.is_empty() {
                // Treat command as a script if a known label is present.
                if get_script_from_label("copyq:", cmd, i, &mut script) {
                    command.extend_from_slice(&["copyq", "eval", "--", &script]);
                } else if get_script_from_label("sh:", cmd, i, &mut script) {
                    command.extend_from_slice(&["sh", "-c", "--", &script, "--"]);
                } else if get_script_from_label("bash:", cmd, i, &mut script) {
                    command.extend_from_slice(&["bash", "-c", "--", &script, "--"]);
                } else if get_script_from_label("perl:", cmd, i, &mut script) {
                    command.extend_from_slice(&["perl", "-e", &script, "--"]);
                } else if get_script_from_label("python:", cmd, i, &mut script) {
                    command.extend_from_slice(&["python", "-c", &script]);
                } else if get_script_from_label("ruby:", cmd, i, &mut script) {
                    command.extend_from_slice(&["ruby", "-e", &script, "--"]);
                }

                if !script.is_empty() {
                    command.extend_from_slice(&captured_texts[1..]);
                    commands.push(command.clone());
                    lines.push(commands.clone());
                    return lines;
                }
            }

            arg.push(c);
        }
    }

    append_and_clear_non_empty(&mut arg, &mut command);
    append_and_clear_non_empty(&mut command, &mut commands);
    append_and_clear_non_empty(&mut commands, &mut lines);

    lines
}

fn pipe_through_processes<T: AsRef<str>>(cmds: Vec<Vec<T>>) {
    let mut iter1 = cmds.iter();
    while let Some(it1) = iter1.next() {
        if let Some(it2) = iter1.next() {
            let mut process1 = Command::new(it1[0].as_ref())
                .args(it1.iter().skip(1).map(AsRef::as_ref))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to start process");

            let mut process2 = Command::new(it2[0].as_ref())
                .args(it2.iter().skip(1).map(AsRef::as_ref))
                .stdin(process1.stdout.take().expect("Failed to open stdout"))
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to start process");

            process1.stdout = Some(process2.stdin.take().expect("Failed to open stdin"));

            processsignals::connect_process_finished(&mut process2, &mut process1, |p| p.terminate());

            process1.wait().expect("Error waiting for process");
            process2.wait().expect("Error waiting for process");
        }
    }
}

fn terminate_process(process: &mut std::process::Child) {
    if let Ok(status) = process.try_wait() {
        if let Some(code) = status.code() {
            if code == 0 {
                return;
            }
        }
    }

    if let Err(_) = process.terminate() {
        if let Err(_) = process.kill() {
            process.kill().expect("Failed to kill process");
        }
    }

    process.wait().expect("Failed to wait for process");
}

struct Action {
    failed: bool,
    current_line: usize,
    exit_code: i32,
    cmds: Vec<Vec<Vec<String>>>,
    processes: Vec<std::process::Child>,
    input: Vec<u8>,
    read_output: bool,
    working_directory_path: Option<String>,
}

impl Action {
    fn new() -> Self {
        Action {
            failed: false,
            current_line: 0,
            exit_code: 0,
            cmds: Vec::new(),
            processes: Vec::new(),
            input: Vec::new(),
            read_output: false,
            working_directory_path: None,
        }
    }

    fn command_line(&self) -> String {
        let mut text = String::new();
        for line in &self.cmds {
            for args in line {
                if !text.is_empty() {
                    text.push('|');
                }
                text.push_str(&args.join(" "));
            }
            text.push('\n');
        }
        text.trim().to_string()
    }

    fn set_command(&mut self, command: &str, arguments: &[String]) {
        self.cmds = parse_commands(command, arguments);
    }

    fn set_input_with_format(&mut self, data: HashMap<String, String>, input_format: &str) {
        if input_format == "mimeItems" {
            self.input = serialize::serialize_data(&data);
            self.read_output = true;
        } else {
            self.input = data.get(input_format).map(|s| s.as_bytes().to_vec()).unwrap_or_default();
            self.read_output = false;
        }
    }

    fn start(&mut self) {
        self.close_sub_commands();

        if self.current_line + 1 >= self.cmds.len() {
            self.finish();
            return;
        }

        self.current_line += 1;
        let cmds = &self.cmds[self.current_line];

        assert!(!cmds.is_empty());

        let mut env = std::collections::HashMap::new();
        if let Some(id) = self.get_id() {
            env.insert("COPYQ_ACTION_ID", id.to_string());
        }
        if let Some(name) = self.get_name() {
            env.insert("COPYQ_ACTION_NAME", name);
        }

        for (i, cmd) in cmds.iter().enumerate() {
            let mut process = Command::new(&cmd[0])
                .args(&cmd[1..])
                .envs(env.iter())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to start process");

            if let Some(working_dir) = &self.working_directory_path {
                process.current_dir(working_dir);
            }

            processsignals::connect_process_error(&mut process, |e| {
                self.on_sub_process_error(e);
            });

            process
                .stdout
                .as_mut()
                .map(|stdout| {
                    processsignals::connect_process_output(stdout, |out| {
                        self.append_output(out);
                    });
                })
                .expect("Failed to connect stdout");

            process
                .stderr
                .as_mut()
                .map(|stderr| {
                    processsignals::connect_process_output(stderr, |err| {
                        self.append_error_output(err);
                    });
                })
                .expect("Failed to connect stderr");

            self.processes.push(process);
        }

        pipe_through_processes(self.processes.iter_mut().collect());

        let last_process = self.processes.last_mut().expect("No last process");
        processsignals::connect_process_started(last_process, || {
            self.on_sub_process_started();
        });
        processsignals::connect_process_finished(last_process, || {
            self.on_sub_process_finished();
        });

        let first_process = self.processes.first_mut().expect("No first process");
        processsignals::connect_process_started(first_process, || {
            self.write_input();
        });
        processsignals::connect_process_bytes_written(first_process, || {
            self.on_bytes_written();
        });

        let need_write = !self.input.is_empty();
        if self.processes.len() == 1 {
            let mode = if need_write && self.read_output {
                std::process::Command::new("").stdin(Stdio::piped()).stdout(Stdio::piped())
            } else if need_write {
                std::process::Command::new("").stdin(Stdio::piped())
            } else if self.read_output {
                std::process::Command::new("").stdout(Stdio::piped())
            } else {
                std::process::Command::new("")
            };
            start_process(&cmds[0], mode).expect("Failed to start process");
        } else {
            let mut iter = self.processes.iter_mut().zip(cmds.iter());
            if let Some((first_process, first_cmd)) = iter.next() {
                start_process(first_cmd, std::process::Command::new(""))
                    .expect("Failed to start process");
                for (process, cmd) in iter {
                    start_process(cmd, std::process::Command::new(""))
                        .expect("Failed to start process");
                }
                start_process(
                    &cmds.last().expect("No last command"),
                    std::process::Command::new(""),
                )
                .expect("Failed to start process");
            }
        }
    }

    fn get_id(&self) -> Option<i32> {
        None
    }

    fn get_name(&self) -> Option<&str> {
        None
    }

    fn wait_for_finished(&mut self, msecs: i32) -> bool {
        if !self.is_running() {
            return true;
        }

        let mut self_ref = Some(self);
        let mut loop_fn = || {
            if let Some(action) = self_ref.as_mut() {
                if msecs >= 0 {
                    let timeout = std::time::Duration::from_millis(msecs as u64);
                    std::thread::sleep(timeout);
                }
                action.on_action_finished();
            }
        };

        while let Some(action) = self_ref.as_mut() {
            if action.is_running() && msecs >= 0 {
                loop_fn();
            } else {
                return true;
            }
        }

        false
    }

    fn is_running(&self) -> bool {
        !self.processes.is_empty() && self.processes.last().unwrap().try_wait().is_none()
    }

    fn set_data(&mut self, data: HashMap<String, String>) {
        self.data = data;
    }

    fn data(&self) -> &HashMap<String, String> {
        &self.data
    }
}