use std::{fs, process::{Child, Command}};
use colored::Colorize;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use serde_json::{from_reader, Value};
mod error;
use error::RustyError;

fn load_config() -> Value {
    let file = fs::File::open("config.json").expect("file should open read only");
    let json_config: Value = from_reader(file).expect("JSON was not well-formatted");
    
    return json_config
}

fn spawn_command(cmd: &str, args: &Vec<&str>) -> Result<Child, RustyError> {
    let mut command = Command::new(cmd);
    for arg in args.iter() {
        command.arg(arg);
    }
    let cmd_result = command.spawn();
    if cmd_result.is_ok() { Ok(cmd_result.unwrap()) }
    else { Err(RustyError) }
}

fn get_pressed_key() -> Result<String, std::io::Error> {    
    loop {
        // Poll for events with a timeout of 1 second
        if event::poll(std::time::Duration::from_secs(0))? {
            // Read the event
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Right | KeyCode::Enter => { return Ok("enter".to_string()); },
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
}

fn print_presentation(title: &str, options: &Vec<Value>, selected_index: usize) {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    println!("{}", title.green().bold());
    for (index, elem) in options.iter().enumerate() {
        if index == selected_index { println!("{} {}", "[X]".purple(), elem.get("name").unwrap().as_str().unwrap().red()) }
        else { println!("{} {}", "[ ]".purple(), elem.get("name").unwrap().as_str().unwrap().red()) }
    }
}

fn handle_song_selection_screen(list_of_songs: &Vec<Value>) -> Result<String, RustyError> {
    let mut selected_song_index = 0;
    loop {
        print_presentation("Select song", list_of_songs, selected_song_index);
        match get_pressed_key().unwrap().as_str() {
            "up" => { if selected_song_index < 1 { selected_song_index = 0 } else { selected_song_index -= 1 } },
            "down" => { if selected_song_index + 1 >= list_of_songs.len() { selected_song_index = list_of_songs.len() - 1 } else { selected_song_index += 1 } },
            "left" => { return Err(RustyError); }
            "enter" => {break;},
            "exit" => { std::process::exit(0) }
            _ => {}
        }
    }

    let selected_song = list_of_songs.get(selected_song_index).ok_or(RustyError).unwrap().get("location").unwrap().as_str().unwrap();
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    println!("{} {}", "Playing".blue().bold(), selected_song);
    return Ok(selected_song.to_string());
}

fn handle_playlist_selection_screen(json_config: &Value) -> Result<String, RustyError> {
    let list_of_playlists = json_config.get("playlists")
    .ok_or(RustyError)?.as_array()
    .ok_or(RustyError)?;
    let mut selected_playlist_index = 0;
    loop {
        print_presentation("Select playlist", list_of_playlists, selected_playlist_index);
        match get_pressed_key().unwrap().as_str() {
            "up" => {
                if selected_playlist_index < 1 { selected_playlist_index = 0 } else { selected_playlist_index -= 1 }
            },
            "down" => {
                if selected_playlist_index + 1 >= list_of_playlists.len() { selected_playlist_index = list_of_playlists.len() - 1 } else { selected_playlist_index += 1 }
            },
            "enter" => {
                let handle_song_selection_screen_result = handle_song_selection_screen(
                    list_of_playlists.get(selected_playlist_index)
                    .ok_or(RustyError)?.get("songs")
                    .ok_or(RustyError)?.as_array()
                    .ok_or(RustyError)?
                );
                if handle_song_selection_screen_result.is_err() { continue }
                else { return handle_song_selection_screen_result }
            },
            "exit" => { std::process::exit(0) }
            _ => {}
        }
    }
}

fn main() {
    let json_config = load_config();

    loop {
        let selected_song = handle_playlist_selection_screen(&json_config);

        if selected_song.is_ok() {
            let mut mpv_child = spawn_command("mpv", 
                &vec![
                    selected_song.unwrap().as_str(), 
                    "--no-video", "--vo=null", "--video=no", 
                    r"--input-ipc-server=\\.\pipe\mpvsocket"
                ]
            ).unwrap();
            
            loop {
                match get_pressed_key().unwrap().as_str() {
                    "exit" => {
                        spawn_command("cmd", &vec!["/C", "echo", "stop", ">",r"\\.\pipe\mpvsocket"]).unwrap();
                        break;
                    },
                    "pause" => {
                        spawn_command("cmd", &vec!["/C", "echo", "cycle", "pause", ">", r"\\.\pipe\mpvsocket"]).unwrap();
                    }
                    _ => {}
                }
            }

            let _exit_status = mpv_child.wait();
        }
    }
}
