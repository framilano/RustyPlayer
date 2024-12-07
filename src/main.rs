use serde_json::{from_reader, Value};
use subprocess::{Popen, PopenConfig, Redirection};
use std::fs;
use std::io::stdin;
use std::thread::{self};
use std::time::Duration;

fn load_config() -> Value {
    let file = fs::File::open("config.json").expect("file should open read only");
    let json_config: Value = from_reader(file).expect("JSON was not well-formatted");
    
    return json_config
}

fn main() {
    let json_config = load_config();

    while (true) {
        let mut input = String::new(); 
        stdin().read_line(&mut input).expect("Failed to read line"); 
        println!("You entered: {}", input.trim());
    }

    let mut p: Popen = Popen::create(&["mpv", "https://www.youtube.com/watch?v=pfiCNAc2AgU", "--no-video"], PopenConfig {
        stdin: Redirection::Pipe, ..Default::default()
    }).expect("Nah");
    
    println!("Hello!");
    
    thread::sleep(Duration::from_secs(5));

    println!("Slept 5 seconds");
    let communication = p.communicate(Some("p\n"));
    println!("{:?}", communication.unwrap());

    // Obtain the output from the standard streams.
    let (out, err) = p.communicate(None).expect("Nahh");
    
    
    print!("Finished")
}
