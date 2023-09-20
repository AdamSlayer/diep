use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;

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


fn main() {
    // connect
    match TcpStream::connect("127.0.0.1:8080") {
        Ok(mut stream) => {

            println!("Successfully connected to server in port 8080");

            loop {
                let msg = b"Hello!";

                // Create some sample data
                let my_data = InputOverNetwork {
                    up_pressed: false,
                    up_just: false,
                    down_pressed: false,
                    down_just: false,
                    left_pressed: false,
                    left_just: false,
                    right_pressed: false,
                    right_just: false,
                    mousepos: (0., 0.),
                };

                // Serialize the data to JSON
                let serialized_data = serde_json::to_string(&my_data).expect("Serialization failed");


                stream.set_nonblocking(false).unwrap();
                // Send the serialized data
                let _ = stream
                    .write_all(serialized_data.as_bytes())
                    .expect("Write failed");


                stream.set_nonblocking(true).unwrap();
                // receive data
                let mut data = [0 as u8; 6]; // using 6 byte buffer
                match stream.read(&mut data) {
                    Ok(_) => {
                        if &data == msg {
                            println!("Reply is ok!");
                        } else {
                            let text = from_utf8(&data).unwrap();
                            println!("Unexpected reply: {}", text);
                        }
                    },
                    Err(e) => {
                        // println!("Failed to receive data: {}", e);
                    }
                }
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}
