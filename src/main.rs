use std::{fs, process::Command};
use colored::Colorize;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use serde_json::{from_reader, Value};

fn load_config() -> Value {
    let file = fs::File::open("config.json").expect("file should open read only");
    let json_config: Value = from_reader(file).expect("JSON was not well-formatted");
    
    return json_config
}

fn get_pressed_key() -> Result<String, std::io::Error> {    
    loop {
        // Poll for events with a timeout of 1 second
        if event::poll(std::time::Duration::from_secs(0)).unwrap() {
            // Read the event
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Right | KeyCode::Enter => { return Ok("enter".to_string()); },
                        KeyCode::Up => { return Ok("up".to_string()); },
                        KeyCode::Down => { return Ok("down".to_string()); },
                        KeyCode::Esc | KeyCode::Char('q') => { std::process::exit(0) }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn handle_song_selection_screen(json_config: &Value) -> &str {
    let list_of_songs = json_config.get("saved_songs").unwrap().as_array().unwrap();
    let mut selected_song_index = 0;
    loop {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        println!("{}", "Select song:".blue().bold());
        for (index, song) in list_of_songs.iter().enumerate() {
            if index == selected_song_index { println!("{} {}", "[X]".purple(), song.as_str().unwrap().red()) }
            else { println!("{} {}", "[ ]".purple(), song.as_str().unwrap().red()) }
        }
        let result = get_pressed_key();
        match result.unwrap().as_str() {
            "up" => {
                if selected_song_index < 1 { selected_song_index = 0 } else { selected_song_index -= 1 }
            },
            "down" => {
                if selected_song_index + 1 >= list_of_songs.len() { selected_song_index = list_of_songs.len() - 1 } else { selected_song_index += 1 }
            },
            "enter" => {break;},
            _ => {}
        }
    }

    let selected_song = list_of_songs.get(selected_song_index).unwrap().as_str().unwrap();
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    println!("{} {}", "Playing".blue().bold(), selected_song);
    return selected_song;
}

fn main() {
    let json_config = load_config();

    loop {
        let selected_song = handle_song_selection_screen(&json_config);
    
        let mut output = Command::new("mpv")
            .arg(selected_song)
            .arg("--no-video")
            .spawn()
            .expect("Failed to execute command");
    
        let _exit_status = output.wait();
    }
}
