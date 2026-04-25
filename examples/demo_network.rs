//! Network smart-home example.
//!
//! Demonstrates a smart home where sockets communicate via TCP and
//! thermometers receive temperature data over UDP.
//!
//! # Running with emulators
//!
//! Open three terminals:
//!
//! ```text
//! # Terminal 1 – socket emulator
//! cargo run --bin socket_emulator -- 127.0.0.1:55001
//!
//! # Terminal 2 – thermometer emulator (uses thermo_emulator.conf by default)
//! cargo run --bin thermo_emulator
//!
//! # Terminal 3 – this example
//! cargo run --example demo_network
//! ```
//!
//! The example works even when emulators are not running: it prints the home
//! report and shows an error message for each unreachable device instead of
//! panicking.

use smart_home::{Report, Room, SmartHome, Socket, Thermometer};
use std::error::Error;
use std::thread;
use std::time::Duration;

const SOCKET_ADDR: &str = "127.0.0.1:55001";
const THERMO_BIND: &str = "127.0.0.1:8890";

/// Prints a labelled report for any type that implements [`Report`].
fn print_report<R: Report>(label: &str, item: &R) {
    let sep = "=".repeat(50);
    println!("\n{sep}\n{label}\n{sep}");
    print!("{}", item.report());
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // ── Build network-connected devices ───────────────────────────────────

    // UDP thermometer: binds a local port and listens for datagrams from the
    // thermo_emulator.  Construction succeeds even without the emulator running;
    // temperature() will return Err(NoDataReceived) until a packet arrives.
    let kitchen_sensor = match Thermometer::new_udp("Kitchen sensor", THERMO_BIND) {
        Ok(t) => {
            println!("Thermometer bound to {THERMO_BIND}, waiting for data…");
            t
        }
        Err(e) => {
            eprintln!("Cannot bind UDP thermometer: {e}");
            // Fall back to a local mock so the home can still be built.
            Thermometer::new("Kitchen sensor (mock)", 0.0)
        }
    };

    // TCP socket: stores the emulator address, connects on each operation.
    let kitchen_lamp = Socket::new_tcp("Kitchen lamp", SOCKET_ADDR);

    // ── Assemble the home ─────────────────────────────────────────────────

    let mut kitchen = Room::new("Kitchen");
    kitchen.add_device("sensor", kitchen_sensor);
    kitchen.add_device("lamp", kitchen_lamp);

    // A local room for comparison.
    let living_room = Room::new("Living room");

    let mut home = SmartHome::new("Network Smart Home");
    home.add_room("kitchen", kitchen);
    home.add_room("living_room", living_room);

    // ── Initial report (errors shown inline) ─────────────────────────────

    println!("\nWaiting 1.5 s for the first UDP packet…");
    thread::sleep(Duration::from_millis(1500));

    print_report("INITIAL HOME STATE", &home);

    // ── Try to control the TCP socket ─────────────────────────────────────

    println!("\n=== Attempting to turn on kitchen lamp via TCP ===");
    if let Some(room) = home.get_room_mut("kitchen") {
        if let Some(lamp) = room.get_device_mut("lamp").and_then(|d| d.as_socket_mut()) {
            match lamp.turn_on() {
                Ok(()) => println!("Kitchen lamp turned ON."),
                Err(e) => println!("Could not turn on kitchen lamp: {e}"),
            }
        }
    }

    // ── Report after toggle ───────────────────────────────────────────────

    print_report("HOME STATE AFTER TOGGLE ATTEMPT", &home);

    // ── Error demonstration ───────────────────────────────────────────────

    println!("\n=== Error handling demo ===");

    match home.get_device("kitchen", "sensor") {
        Ok(d) => println!("Sensor report: {}", d.report()),
        Err(e) => println!("Error: {e}"),
    }

    match home.get_device("nonexistent_room", "lamp") {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected error: {e}"),
    }

    Ok(())
}
