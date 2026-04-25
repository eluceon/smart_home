//! UDP thermometer emulator.
//!
//! Reads a target UDP address and a send interval from a configuration file,
//! then periodically sends a simulated temperature value as a UTF-8 string.
//!
//! # Usage
//!
//! ```text
//! cargo run --bin thermo_emulator -- thermo_emulator.conf
//! ```
//!
//! If no path is given the default `thermo_emulator.conf` is used.
//!
//! # Configuration file format
//!
//! Plain text, two lines:
//!
//! ```text
//! 127.0.0.1:8890
//! 1000
//! ```
//!
//! Line 1 — UDP destination address (`host:port`).
//! Line 2 — Send interval in milliseconds (must be > 0).
//!
//! # Protocol
//!
//! Each datagram is a UTF-8 encoded decimal float followed by a newline, e.g.
//! `"23.5\n"`.  The [`Thermometer::new_udp`][smart_home::Thermometer] receiver
//! parses this format.

use std::io;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "thermo_emulator.conf".to_string());

    let (dest_addr, interval_ms) = read_config(&config_path)?;

    if interval_ms == 0 {
        anyhow::bail!("interval must be > 0 ms");
    }

    let interval = Duration::from_millis(interval_ms);

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_nonblocking(true)?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || r.store(false, Ordering::Release))?;

    log::info!("Thermometer emulator → {dest_addr} every {interval_ms} ms  (press Ctrl+C to stop)");

    while running.load(Ordering::Acquire) {
        let temp = simulate_temperature();
        let payload = format!("{temp:.1}\n");

        match socket.send_to(payload.as_bytes(), &dest_addr) {
            Ok(_) => log::debug!("sent {temp:.1} °C to {dest_addr}"),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                log::warn!("send would block, skipping");
            }
            Err(e) => {
                log::error!("send error: {e}");
                return Err(e.into());
            }
        }

        thread::sleep(interval);
    }

    log::info!("Thermometer emulator shutting down");
    Ok(())
}

/// Returns a pseudo-random temperature in the range [15.0, 35.0] °C.
///
/// The value is derived from the current time, giving a slowly varying but
/// deterministic-looking stream of measurements.
fn simulate_temperature() -> f32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    // Scale to [0.0, 20.0] and shift to [15.0, 35.0].
    15.0 + (nanos % 2001) as f32 / 100.0
}

fn read_config(path: &str) -> anyhow::Result<(String, u64)> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("cannot read config file '{path}': {e}"))?;

    let mut lines = content.lines();

    let addr = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("config is missing the address line"))?
        .trim()
        .to_string();

    let interval_ms: u64 = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("config is missing the interval line"))?
        .trim()
        .parse()
        .map_err(|e| anyhow::anyhow!("interval must be a positive integer (ms): {e}"))?;

    Ok((addr, interval_ms))
}
