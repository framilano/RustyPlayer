use std::{env::current_exe, fs::{self, File}, io::stdout, process::{Child, Command}};
use crossterm::{cursor::{MoveLeft, MoveTo}, event::{self, Event, KeyCode, KeyEventKind}, execute, style::Stylize, terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType}};
use serde_json::{from_reader, Value};
use crate::error::RustyError;

pub fn load_config() -> Result<Value, RustyError> {
    let current_path = current_exe().unwrap();
    let executable_path = current_path.to_str().unwrap();
    
    let file: Result<File, std::io::Error>;
    //Running program from vscode
    if executable_path.contains("/target/debug") || executable_path.contains("\\target\\debug") {
        file = fs::File::open("config.json");
        if file.is_err() { return Err(RustyError); }
    } else {
    //Running program from compiled executable
        let og_len = executable_path.len();
        let mut executable_path_str = executable_path.to_string();
        match std::env::consts::OS {
            "windows" => {
                executable_path_str.truncate(og_len - "\\rusty-player.exe".len());
                executable_path_str += "\\"
            },
            "linux" | "macos" => {
                executable_path_str.truncate(og_len - "/rusty-player".len());
                executable_path_str += "/"
            },
            _ => {
                print!("Invalid env")
            }
        }
        file = fs::File::open(format!("{}config.json", executable_path_str));
        if file.is_err() { return Err(RustyError); }
    }
    let json_config: Value = from_reader(file.unwrap()).expect("JSON was not well-formatted");
    
    return Ok(json_config)
}

pub fn spawn_command(cmd: &str, args: &Vec<&str>) -> Result<Child, RustyError> {
    let mut command: Command;
    match std::env::consts::OS {
        "windows" => {
            command = Command::new("cmd");
            command.arg("/C");
            command.arg(cmd);
        },
        "linux" | "macos" => {
            command = Command::new(cmd);
        },
        _ => return Err(RustyError),
    }

    for arg in args.iter() {
        command.arg(arg);
    }
    let cmd_result = command.spawn();
    if cmd_result.is_ok() { Ok(cmd_result.unwrap()) }
    else { Err(RustyError) }
}

pub fn get_pressed_key() -> Result<String, std::io::Error> {    
    
    let _ = enable_raw_mode();
    loop {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind == KeyEventKind::Press {
                let value: String = match key_event.code {
                    KeyCode::Enter => "enter".to_string(),
                    KeyCode::Right => "right".to_string(),
                    KeyCode::Up => "up".to_string(),
                    KeyCode::Down => "down".to_string(),
                    KeyCode::Left => "left".to_string(),
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('c') => "exit".to_string(),
                    KeyCode::Char('p') => "pause".to_string(),
                    _ => {continue}
                };

                let _ = disable_raw_mode();
                return Ok(value);
            }
        }
    }
}

pub fn clear_screen() {
    if std::env::consts::OS == "windows" { spawn_command("cls", &vec![]).unwrap().wait().unwrap(); }
    else { execute!(stdout(), Clear(ClearType::All)).unwrap(); }
    execute!(stdout(), MoveTo(0, 0)).unwrap();
}

pub fn print_presentation(title: &str, options: &Vec<Value>, selected_index: usize) {
    clear_screen();
    println!("{}", title);
    for (index, elem) in options.iter().enumerate() {
        execute!(stdout(), MoveLeft(100)).unwrap();
        if index == selected_index { println!("{} {}", "[X]".dark_blue(), elem.get("name").unwrap().as_str().unwrap().dark_red().bold()) }
        else { println!("{} {}", "[ ]".dark_blue(), elem.get("name").unwrap().as_str().unwrap().dark_red()) }
    }
}