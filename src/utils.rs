use std::{env::current_exe, fs::{self, File}, io::stdout, process::{Child, Command}};
use crossterm::{cursor::{MoveLeft, MoveTo}, event::{self, Event, KeyCode, KeyEventKind}, execute, style::Stylize, terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType}};
use serde_json::{from_reader, Value};
use crate::error::RustyError;

pub fn load_config() -> Value {
    let current_path = current_exe().unwrap();
    let executable_path = current_path.to_str().unwrap();
    
    let file: File;
    if executable_path.contains("/target/debug") {
        file = fs::File::open("config.json").expect("file should open read only");
    } else {
        let og_len = executable_path.len();
        let mut executable_path_str = executable_path.to_string();
        executable_path_str.truncate(og_len - "/rusty-player".len());
        file = fs::File::open(format!("{}/config.json", executable_path_str)).expect("file should open read only");
    }
    let json_config: Value = from_reader(file).expect("JSON was not well-formatted");
    
    return json_config
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
    
    loop {
        let _ = enable_raw_mode();
        if let Event::Key(key_event) = event::read()? {
            let _ = disable_raw_mode();
            if key_event.kind == KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Enter => { return Ok("enter".to_string()); },
                    KeyCode::Right => { return Ok("right".to_string()); },
                    KeyCode::Up => { return Ok("up".to_string()) },
                    KeyCode::Down => { return Ok("down".to_string()) },
                    KeyCode::Left => { return Ok("left".to_string()) },
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('c') => { return Ok("exit".to_string()) },
                    KeyCode::Char('p') => { return Ok("pause".to_string()) }
                    _ => {
                        //println!("Pressed {}", key_event.code)
                    }
                }
            }
        }
    }
}

pub fn print_presentation(title: &str, options: &Vec<Value>, selected_index: usize) {
    execute!(stdout(), Clear(ClearType::All)).unwrap();
    execute!(stdout(), MoveTo(0, 0)).unwrap();
    println!("{}", title);
    for (index, elem) in options.iter().enumerate() {
        execute!(stdout(), MoveLeft(100)).unwrap();
        if index == selected_index { println!("{} {}", "[X]".dark_blue(), elem.get("name").unwrap().as_str().unwrap().dark_red().bold()) }
        else { println!("{} {}", "[ ]".dark_blue(), elem.get("name").unwrap().as_str().unwrap().dark_red()) }
    }
}