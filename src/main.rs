use crossterm::style::Stylize;
use serde_json::Value;
mod error;
use error::RustyError;
use utils::{get_pressed_key, load_config, print_presentation, spawn_command};
mod utils;

fn handle_song_selection_screen(list_of_songs: &Vec<Value>) -> Result<usize, RustyError> {
    let mut selected_song_index = 0;
    loop {
        print_presentation(&"Select the starting track".bold().magenta().to_string(), list_of_songs, selected_song_index);
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
            "exit" => { std::process::exit(0);}
            _ => {
                print!("Default")
            }
        }
    }
}

fn create_playlist(current_track: usize, current_cd: &Vec<Value>) -> Vec<&str> {
    let mut playlist: Vec<&str> = vec![];

    //Adding all songs from current_track to the end
    for i in current_track..current_cd.len() {
        playlist.push(current_cd[i].get("location").unwrap().as_str().unwrap());
    }

    //Adding all songs from the beginning to current track
    for i in 0..current_track {
        playlist.push(current_cd[i].get("location").unwrap().as_str().unwrap());
    }

    return playlist;
}


fn play_user_selection(user_selection: &(usize, &Vec<Value>)) {
    let current_track = user_selection.0;
    let current_cd = user_selection.1;
    //let song_location= current_cd.get(current_track).unwrap().get("location").unwrap().as_str().unwrap();
    let song_name;
    let mut mpv_child;    

    let playlist = create_playlist(current_track, current_cd);


    song_name = current_cd.get(current_track).unwrap().get("name").unwrap().as_str().unwrap();
    
    let mut args = vec![
        "--volume=100",
        "--no-video", "--vo=null", "--video=no", "--loop-playlist=inf"
    ];
    args.extend(playlist);
    mpv_child = spawn_command("mpv", &args).unwrap();
    
    print_presentation(&format!("{} {}", "Playing".red().bold(), song_name.bold()), &vec![], 0);
    
    //Waiting for MPV process to stop (q, left, right pressed or the song just finished normally)
    mpv_child.wait().unwrap();

        
    //No key has been pressed, the song finished normally
    //current_track += 1;
    //current_track = current_track % current_cd.len();
        
}



fn main() {

    let json_config = load_config();
    if json_config.is_err() {
        println!("Missing config file next to rust-player executable...");
        return;
    }

    let config_file = json_config.unwrap();

    let mut user_selection_result;
    loop {
        user_selection_result = handle_playlist_selection_screen(&config_file);

        if user_selection_result.is_ok() {
            play_user_selection(&user_selection_result.unwrap());
        }
    }
}
