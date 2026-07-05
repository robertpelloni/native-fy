use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc::Sender;
use std::thread;
use crate::runtime::UiCommand;
use serde_json::Value;

pub fn start_python_bridge(tx: Sender<UiCommand>) {
    thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:9090").expect("Failed to bind Python bridge socket");
        println!("Python IPC Bridge listening on 127.0.0.1:9090");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let tx_clone = tx.clone();
                    thread::spawn(move || handle_client(stream, tx_clone));
                }
                Err(e) => eprintln!("Failed to accept python connection: {}", e),
            }
        }
    });
}

fn handle_client(mut stream: TcpStream, tx: Sender<UiCommand>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();

    while let Ok(bytes) = reader.read_line(&mut line) {
        if bytes == 0 { break; } // Connection closed

        if let Ok(json) = serde_json::from_str::<Value>(&line)
            && let Some(cmd) = json.get("command").and_then(|v| v.as_str()) {
                match cmd {
                    "reload" => {
                        let _ = tx.send(UiCommand::Reload);
                    }
                    "toggleDashboard" => {
                        let _ = tx.send(UiCommand::ToggleDashboard);
                    }
                    "screenshot" => {
                        if let Some(path) = json.get("path").and_then(|v| v.as_str()) {
                            let _ = tx.send(UiCommand::Screenshot { path: path.to_string() });
                        }
                    }
                    _ => println!("Unknown Python command: {}", cmd),
                }
            }

        let _ = stream.write_all(b"{\"status\":\"ok\"}\n");
        line.clear();
    }
}
