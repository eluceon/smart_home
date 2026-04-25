//! Smart socket — local mock and TCP-connected variants.

use crate::error::NetworkError;
use std::io::{BufRead, BufReader, Write as IoWrite};
use std::net::TcpStream;
use std::time::Duration;

/// TCP command: turn the socket on.
pub const CMD_TURN_ON: &str = "TURN_ON";
/// TCP command: turn the socket off.
pub const CMD_TURN_OFF: &str = "TURN_OFF";
/// TCP command: query whether the socket is on (`ON` / `OFF` response).
pub const CMD_STATUS: &str = "STATUS";
/// TCP command: query current power draw (float response, watts).
pub const CMD_POWER: &str = "POWER";

const TCP_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
enum SocketBackend {
    Local { is_on: bool, power_watts: f32 },
    Tcp { addr: String },
}

/// A smart socket that can operate locally (mock) or via a remote TCP server.
///
/// Use [`Socket::new`] for in-process testing and [`Socket::new_tcp`] to
/// communicate with a running [`socket_emulator`].
#[derive(Debug)]
pub struct Socket {
    name: String,
    backend: SocketBackend,
}

impl Socket {
    /// Creates a local (in-process) socket with the given nominal power in watts.
    ///
    /// The socket starts in the **off** state.
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::Socket;
    ///
    /// let socket = Socket::new("Desk lamp", 60.0);
    /// assert!(!socket.is_on().unwrap());
    /// ```
    pub fn new(name: impl Into<String>, power_watts: f32) -> Self {
        Self {
            name: name.into(),
            backend: SocketBackend::Local {
                is_on: false,
                power_watts,
            },
        }
    }

    /// Creates a TCP-connected socket that delegates all operations to a remote
    /// [`socket_emulator`] listening at `addr`.
    ///
    /// No connection is established until one of the operation methods is called.
    pub fn new_tcp(name: impl Into<String>, addr: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            backend: SocketBackend::Tcp { addr: addr.into() },
        }
    }

    /// Returns the socket name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Turns the socket on.
    ///
    /// # Errors
    ///
    /// Returns [`NetworkError`] if the TCP connection fails or the server
    /// returns an unexpected response.
    #[must_use = "turning the socket on may fail; check the Result"]
    pub fn turn_on(&mut self) -> Result<(), NetworkError> {
        match &mut self.backend {
            SocketBackend::Local { is_on, .. } => {
                *is_on = true;
                Ok(())
            }
            SocketBackend::Tcp { addr } => send_cmd(addr, CMD_TURN_ON),
        }
    }

    /// Turns the socket off.
    ///
    /// # Errors
    ///
    /// Returns [`NetworkError`] if the TCP connection fails or the server
    /// returns an unexpected response.
    #[must_use = "turning the socket off may fail; check the Result"]
    pub fn turn_off(&mut self) -> Result<(), NetworkError> {
        match &mut self.backend {
            SocketBackend::Local { is_on, .. } => {
                *is_on = false;
                Ok(())
            }
            SocketBackend::Tcp { addr } => send_cmd(addr, CMD_TURN_OFF),
        }
    }

    /// Returns `true` if the socket is currently on.
    ///
    /// # Errors
    ///
    /// Returns [`NetworkError`] if the TCP connection fails or the response
    /// cannot be parsed.
    pub fn is_on(&self) -> Result<bool, NetworkError> {
        match &self.backend {
            SocketBackend::Local { is_on, .. } => Ok(*is_on),
            SocketBackend::Tcp { addr } => {
                let resp = query(addr, CMD_STATUS)?;
                match resp.as_str() {
                    "ON" => Ok(true),
                    "OFF" => Ok(false),
                    other => Err(NetworkError::Protocol(format!(
                        "unexpected STATUS response: '{other}'"
                    ))),
                }
            }
        }
    }

    /// Returns the current power draw in watts.
    ///
    /// Returns `0.0` when the socket is off, the nominal wattage when on.
    ///
    /// # Errors
    ///
    /// Returns [`NetworkError`] if the TCP connection fails or the response
    /// cannot be parsed as a float.
    pub fn power(&self) -> Result<f32, NetworkError> {
        match &self.backend {
            SocketBackend::Local {
                is_on, power_watts, ..
            } => {
                if *is_on {
                    Ok(*power_watts)
                } else {
                    Ok(0.0)
                }
            }
            SocketBackend::Tcp { addr } => {
                let resp = query(addr, CMD_POWER)?;
                resp.parse::<f32>()
                    .map_err(|e| NetworkError::Protocol(e.to_string()))
            }
        }
    }
}

// ── TCP helpers ───────────────────────────────────────────────────────────────

/// Opens a TCP connection, sends `cmd`, reads the response line, and checks it
/// equals `"OK"`.
fn send_cmd(addr: &str, cmd: &str) -> Result<(), NetworkError> {
    let resp = query(addr, cmd)?;
    if resp == "OK" {
        Ok(())
    } else {
        Err(NetworkError::Protocol(format!(
            "expected OK for {cmd}, got '{resp}'"
        )))
    }
}

/// Opens a TCP connection, sends `cmd`, and returns the trimmed response line.
fn query(addr: &str, cmd: &str) -> Result<String, NetworkError> {
    let stream = TcpStream::connect(addr)?;
    stream.set_read_timeout(Some(TCP_TIMEOUT))?;
    stream.set_write_timeout(Some(TCP_TIMEOUT))?;

    let mut writer = &stream;
    writeln!(writer, "{cmd}")?;

    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line.trim().to_string())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_creation() {
        let socket = Socket::new("Air conditioner", 1500.0);
        assert!(!socket.is_on().unwrap());
        assert_eq!(socket.power().unwrap(), 0.0);
        assert_eq!(socket.name(), "Air conditioner");
    }

    #[test]
    fn test_socket_turn_on_off() {
        let mut socket = Socket::new("Kettle", 2000.0);

        socket.turn_on().unwrap();
        assert!(socket.is_on().unwrap());
        assert_eq!(socket.power().unwrap(), 2000.0);

        socket.turn_off().unwrap();
        assert!(!socket.is_on().unwrap());
        assert_eq!(socket.power().unwrap(), 0.0);
    }
}
