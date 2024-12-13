use std::{sync::mpsc::{self, Sender}, thread};
use crossterm::style::Stylize;
use serde_json::Value;
mod error;
use error::RustyError;
use utils::{get_pressed_key, load_config, print_presentation, spawn_command};
mod utils;

fn handle_song_selection_screen(list_of_songs: &Vec<Value>) -> Result<usize, RustyError> {
    let mut selected_song_index = 0;
    loop {
        print_presentation("Select the starting track", list_of_songs, selected_song_index);
        match get_pressed_key().unwrap().as_str() {
            "up" => { if selected_song_index < 1 { selected_song_index = 0 } else { selected_song_index -= 1 } },
            "down" => { if selected_song_index + 1 >= list_of_songs.len() { selected_song_index = list_of_songs.len() - 1 } else { selected_song_index += 1 } },
            "left" => { return Err(RustyError); }
            "enter" | "right" => {break;},
            "exit" => { std::process::exit(0) }
            _ => {}
        }
    }

    return Ok(selected_song_index);
}

fn handle_playlist_selection_screen(json_config: &Value) -> Result<(usize, &Vec<Value>), RustyError> {
    let list_of_playlists = json_config.get("cds")
    .ok_or(RustyError)?.as_array()
    .ok_or(RustyError)?;
    let mut selected_playlist_index = 0;
    loop {
        print_presentation(&"Choose CD".bold().magenta().to_string(), list_of_playlists, selected_playlist_index);
        match get_pressed_key().unwrap().as_str() {
            "up" => {
                if selected_playlist_index < 1 { selected_playlist_index = 0 } else { selected_playlist_index -= 1 }
            },
            "down" => {
                if selected_playlist_index + 1 >= list_of_playlists.len() { selected_playlist_index = list_of_playlists.len() - 1 } else { selected_playlist_index += 1 }
            },
            "enter" | "right" => {
                let list_of_songs = list_of_playlists.get(selected_playlist_index)
                    .ok_or(RustyError)?.get("songs")
                    .ok_or(RustyError)?.as_array()
                    .ok_or(RustyError)?;
                let handle_song_selection_screen_result = handle_song_selection_screen(list_of_songs);
                if handle_song_selection_screen_result.is_err() { continue }
                else { return Ok((handle_song_selection_screen_result.unwrap(), list_of_songs)) }
            },
            "exit" => { std::process::exit(0) }
            _ => {}
        }
    }
}

fn handle_mpv_controls(tx: &Sender<&str>) {
    let mut pressed_key;
    loop {
        pressed_key = get_pressed_key().unwrap();
        match pressed_key.as_str() {
            "exit" => {
                tx.send("exit").unwrap();
                spawn_command("cmd", &vec!["/C", "echo", "stop", ">", r"\\.\pipe\mpvsocket"]).unwrap();
                return
            },
            "left" => {
                tx.send("left").unwrap();
                spawn_command("cmd", &vec!["/C", "echo", "stop", ">", r"\\.\pipe\mpvsocket"]).unwrap();
            }, 
            "right" => {
                tx.send("right").unwrap();
                spawn_command("cmd", &vec!["/C", "echo", "stop", ">", r"\\.\pipe\mpvsocket"]).unwrap();
            },
            "pause" => {
                //pause doesn't need to tell the main thread anything, it's only communicating with MPV
                spawn_command("cmd", &vec!["/C", "echo", "cycle", "pause", ">", r"\\.\pipe\mpvsocket"]).unwrap();
            },
            _ => {}
        }  
    }
}

fn play_user_selection(user_selection: &(usize, &Vec<Value>)) {
    let mut current_track = user_selection.0;
    let current_cd = user_selection.1;
    let mut song_location;
    let mut song_name;
    let mut mpv_child;
    let mut possible_thread_msg;
    
    
    //Creating a channel between the key handling thread and the main thread
    let (tx, rx) = mpsc::channel();
   
    //Why a separate thread? We need to check when a song ends normally, not only when a key is pressed like in the menus
    thread::spawn(move || handle_mpv_controls(&tx));

    loop {
        song_location = current_cd.get(current_track).unwrap().get("location").unwrap().as_str().unwrap();
        song_name = current_cd.get(current_track).unwrap().get("name").unwrap().as_str().unwrap();
        
        mpv_child = spawn_command("mpv", 
        &vec![
            song_location, "--volume=100",
            "--no-video", "--vo=null", "--video=no", 
            r"--input-ipc-server=\\.\pipe\mpvsocket"
            ]
        ).unwrap();
        
        print_presentation(&format!("{} {}", "Playing".red().bold(), song_name.bold()), &vec![], 0);
        
        //Waiting for MPV process to stop (q, left, right pressed or the song just finished normally)
        mpv_child.wait().unwrap();
        
        //If a message has been received from the key handling thread, then a key has been pressed
        possible_thread_msg = rx.try_recv();
        if possible_thread_msg.is_ok() {
            match possible_thread_msg.unwrap() {
                "exit" => {return},
                "left" => {
                    if current_track == 0 { current_track = 1}
                    current_track -= 1;
                    
                },
                "right" => {
                    current_track += 1;
                    current_track = current_track % current_cd.len();
                }
                _ => {}
            }
        } else {
            //No key has been pressed, the song finished normally
            current_track += 1;
            current_track = current_track % current_cd.len();
        }
    }
}

fn handle_closure() {
    ctrlc::set_handler(move || {
        spawn_command("cmd", &vec!["/C", "echo", "stop", ">", r"\\.\pipe\mpvsocket"]).unwrap();
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");
}

fn main() {
    handle_closure();

    let json_config = load_config();
    let mut user_selection_result;
    loop {
        user_selection_result = handle_playlist_selection_screen(&json_config);

        if user_selection_result.is_ok() {
            play_user_selection(&user_selection_result.unwrap());
        }
    }
}
