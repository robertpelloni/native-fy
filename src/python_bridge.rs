use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc::Sender;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use crate::runtime::UiCommand;
use serde_json::Value;

static SERVER_RUNNING: AtomicBool = AtomicBool::new(true);

pub fn is_running() -> bool {
    SERVER_RUNNING.load(Ordering::SeqCst)
}

pub fn stop_python_bridge() {
    SERVER_RUNNING.store(false, Ordering::SeqCst);
    let _ = TcpStream::connect("127.0.0.1:9090");
    println!("Python IPC Bridge: Server stopped.");
}

pub fn start_python_bridge(tx: Sender<UiCommand>) {
    SERVER_RUNNING.store(true, Ordering::SeqCst);
    thread::spawn(move || {
        let listener = match TcpListener::bind("127.0.0.1:9090") {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to bind Python bridge socket: {}", e);
                return;
            }
        };
        println!("Python IPC Bridge listening on 127.0.0.1:9090");

        for stream in listener.incoming() {
            if !SERVER_RUNNING.load(Ordering::SeqCst) {
                break;
            }
            match stream {
                Ok(stream) => {
                    let tx_clone = tx.clone();
                    thread::spawn(move || handle_client(stream, tx_clone));
                }
                Err(e) => {
                    if SERVER_RUNNING.load(Ordering::SeqCst) {
                        eprintln!("Failed to accept python connection: {}", e);
                    }
                }
            }
        }
    });
}

fn handle_client(mut stream: TcpStream, tx: Sender<UiCommand>) {
    let mut reader = BufReader::new(match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    });
    let mut line = String::new();

    while let Ok(bytes) = reader.read_line(&mut line) {
        if bytes == 0 { break; }

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
                    "shutdown" | "quit" => {
                        let _ = tx.send(UiCommand::ShutdownServers);
                    }
                    _ => println!("Unknown Python command: {}", cmd),
                }
            }

        let _ = stream.write_all(b"{\"status\":\"ok\"}\n");
        line.clear();
    }
}
