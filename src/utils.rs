use std::{fs, process::{Child, Command}};
use crossterm::{event::{self, Event, KeyCode, KeyEventKind}, style::Stylize};
use serde_json::{from_reader, Value};
use crate::error::RustyError;

pub fn load_config() -> Value {
    let file = fs::File::open("config.json").expect("file should open read only");
    let json_config: Value = from_reader(file).expect("JSON was not well-formatted");
    
    return json_config
}

pub fn spawn_command(cmd: &str, args: &Vec<&str>) -> Result<Child, RustyError> {
    let mut command = Command::new(cmd);
    for arg in args.iter() {
        command.arg(arg);
    }
    let cmd_result = command.spawn();
    if cmd_result.is_ok() { Ok(cmd_result.unwrap()) }
    else { Err(RustyError) }
}

pub fn get_pressed_key() -> Result<String, std::io::Error> {    
    loop {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind == KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Enter => { return Ok("enter".to_string()); },
                    KeyCode::Right => { return Ok("right".to_string()); },
                    KeyCode::Up => { return Ok("up".to_string()) },
                    KeyCode::Down => { return Ok("down".to_string()) },
                    KeyCode::Left => { return Ok("left".to_string()) },
                    KeyCode::Esc | KeyCode::Char('q') => { return Ok("exit".to_string()) }
                    KeyCode::Char('p') => { return Ok("pause".to_string()) }
                    _ => {}
                }
            }
        }
    }
}

pub fn print_presentation(title: &str, options: &Vec<Value>, selected_index: usize) {
    spawn_command("cmd", &vec!["/C", "cls"]).unwrap().wait().unwrap();    
    println!("{}", title);
    for (index, elem) in options.iter().enumerate() {
        if index == selected_index { println!("{} {}", "[X]".dark_blue(), elem.get("name").unwrap().as_str().unwrap().dark_red().bold()) }
        else { println!("{} {}", "[ ]".dark_blue(), elem.get("name").unwrap().as_str().unwrap().dark_red()) }
    }
}