//! TCP smart-socket emulator.
//!
//! Listens for TCP connections and emulates a smart socket by responding to
//! text commands.  Multiple clients can connect simultaneously; the socket
//! state is shared across all connections via `Arc<Mutex<_>>`.
//!
//! # Usage
//!
//! ```text
//! cargo run --bin socket_emulator -- 127.0.0.1:55001
//! ```
//!
//! # Protocol
//!
//! Each command is a UTF-8 line terminated by `\n`.  The server responds with
//! a single line terminated by `\n`.
//!
//! | Command    | Response           |
//! |------------|--------------------|
//! | `TURN_ON`  | `OK`               |
//! | `TURN_OFF` | `OK`               |
//! | `STATUS`   | `ON` or `OFF`      |
//! | `POWER`    | float (e.g. `60`)  |

use smart_home::devices::socket::{CMD_POWER, CMD_STATUS, CMD_TURN_OFF, CMD_TURN_ON};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct SocketState {
    is_on: bool,
    power_watts: f32,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:55001".to_string());

    let listener = TcpListener::bind(&addr)?;
    // Non-blocking so the accept loop can check for shutdown without hanging.
    listener.set_nonblocking(true)?;

    let state = Arc::new(Mutex::new(SocketState {
        is_on: false,
        power_watts: 100.0,
    }));

    log::info!("Socket emulator listening on {addr}");
    println!("Socket emulator listening on {addr}  (press Ctrl+C to stop)");

    loop {
        match listener.accept() {
            Ok((stream, peer)) => {
                log::info!("Client connected: {peer}");
                let state_ref = state.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_client(stream, state_ref) {
                        log::warn!("Client handler error: {e}");
                    }
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No pending connection; yield briefly before polling again.
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                log::error!("Accept error: {e}");
                break;
            }
        }
    }

    Ok(())
}

fn handle_client(
    mut stream: TcpStream,
    state: Arc<Mutex<SocketState>>,
) -> Result<(), std::io::Error> {
    let peer = stream.peer_addr().ok();
    let reader = BufReader::new(stream.try_clone()?);

    for line in reader.lines() {
        let cmd = line?;
        let response = process_command(cmd.trim(), &state);
        log::debug!("peer={peer:?} cmd={} response={response}", cmd.trim());
        writeln!(stream, "{response}")?;
    }

    log::info!("Client disconnected: {peer:?}");
    Ok(())
}

fn process_command(cmd: &str, state: &Arc<Mutex<SocketState>>) -> String {
    let mut guard = match state.lock() {
        Ok(g) => g,
        Err(_) => return "ERROR lock poisoned".to_string(),
    };

    if cmd == CMD_TURN_ON {
        guard.is_on = true;
        "OK".to_string()
    } else if cmd == CMD_TURN_OFF {
        guard.is_on = false;
        "OK".to_string()
    } else if cmd == CMD_STATUS {
        if guard.is_on {
            "ON".to_string()
        } else {
            "OFF".to_string()
        }
    } else if cmd == CMD_POWER {
        let power = if guard.is_on { guard.power_watts } else { 0.0 };
        power.to_string()
    } else {
        format!("ERROR unknown command '{cmd}'")
    }
}
