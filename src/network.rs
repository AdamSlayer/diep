use rand::prelude::*;
use std::fs::File;
use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct InputOverNetwork {
    up_pressed: bool,
    up_just: bool,
    down_pressed: bool,
    down_just: bool,
    left_pressed: bool,
    left_just: bool,
    right_pressed: bool,
    right_just: bool,
    mousepos: (f64, f64),
}



// spawned for each connection once, connection ends when the function finishes
fn handle_client(mut stream: TcpStream) {
    let connection_uuid: u128 = thread_rng().gen();
    let mut buffer = [0; 1024];
    let mut data = Vec::new();

    loop {
        let bytes_read = stream.read(&mut buffer).unwrap();
        data.extend_from_slice(&buffer[..bytes_read]);
    
        // Trim the data, and check if the data contains a complete JSON message
        let data_string = String::from_utf8(data.clone()).unwrap();
        let mut inside_how_many_brackets = 0;
        let mut trimmed_data_json = "".to_string();
        for c in data_string.chars() {
            if c == '{' {
                inside_how_many_brackets += 1;
            }
            if c == '}' {
                inside_how_many_brackets -= 1;
            }
            trimmed_data_json.push(c);
            if inside_how_many_brackets == 0 {
                break
            }
        }
        let received_data = serde_json::from_str::<InputOverNetwork>(&trimmed_data_json);
        if received_data.is_ok() {
            let received_data = received_data.unwrap();
            println!("Received data: {:?}", received_data);

            // get the system temp dir path
            let temp_dir = std::env::temp_dir();
            // all paths of received data begin with "recv"
            let file_path = temp_dir.join(format!{"recv{}", connection_uuid});

            // Write received data to a file.
            println!("writing received data to file: {:?}", file_path);
            let mut file = File::create(&file_path).unwrap();
            file.write_all(format!{"{:?}", trimmed_data_json}.as_bytes()).unwrap();
    
            // Clear the data buffer for the next message
            data.clear();
            buffer = [0; 1024];
        } else {
            println!("data: {:?}", trimmed_data_json);
            println!("error: {:?}", received_data);
        }
    }
}

pub fn run_network() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 8080");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move|| {
                    // connection succeeded
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);
}
