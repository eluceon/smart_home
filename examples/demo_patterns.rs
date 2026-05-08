//! Demonstration of the patterns from Assignment 4:
//! HomeBuilder (typestate), Reporter (static polymorphism), Observer (dynamic).
//!
//! Run with:
//!   cargo run --example demo_patterns

use smart_home::{
    HomeBuilder, Report, Reporter, Room, SmartDevice, Socket, Subscriber, Thermometer,
};
use std::error::Error;

/// Helper: prints a labelled report via the `Report` trait.
fn print_report<R: Report>(label: &str, item: &R) {
    let sep = "=".repeat(50);
    println!("\n{sep}\n{label}\n{sep}");
    print!("{}", item.report());
}

fn main() -> Result<(), Box<dyn Error>> {
    // ═══════════════════════════════════════════════════════════════════════
    // 1. HomeBuilder — typestate pattern
    // ═══════════════════════════════════════════════════════════════════════
    println!("\n═══ HomeBuilder (typestate pattern) ═══");

    let mut home = HomeBuilder::new()
        .add_room("Living room")
        .add_device("lamp", Socket::new("Desk lamp", 60.0))
        .add_device("sensor", Thermometer::new("Temp sensor", 22.5))
        .add_room("Bedroom")
        .add_device("heater", Socket::new("Space heater", 2000.0))
        .add_device("night_lamp", Socket::new("Night lamp", 10.0))
        .build();

    // Demonstrate compile-time safety:
    // The following would NOT compile — cannot add device before a room:
    //   HomeBuilder::new().add_device("x", Socket::default());
    //   HomeBuilder::new().build(); // also blocked

    print_report("HOME BUILT WITH BUILDER", &home);

    // ═══════════════════════════════════════════════════════════════════════
    // 2. Observer pattern on a Room
    // ═══════════════════════════════════════════════════════════════════════
    println!("\n═══ Observer pattern ═══");

    let mut room = Room::default();

    // Subscribe a struct implementing the Subscriber trait.
    room.subscribe(PrintSubscriber);

    // Subscribe a closure.
    room.subscribe(|device: &SmartDevice| {
        println!(
            "  [closure subscriber] Device added: {}",
            device.report().lines().next().unwrap_or("")
        );
    });

    println!("Adding devices to room (subscribers will fire):");
    room.add_device("socket_1", Socket::new("Ceiling light", 100.0));
    room.add_device("thermo_1", Thermometer::new("Wall sensor", 23.0));

    // ═══════════════════════════════════════════════════════════════════════
    // 3. Reporter — static polymorphism
    // ═══════════════════════════════════════════════════════════════════════
    println!("\n═══ Reporter (static polymorphism) ═══");

    let socket = Socket::new("Table lamp", 40.0);
    let thermo = Thermometer::new("Hall sensor", 19.0);

    // Build a heterogeneous report at compile time.
    let report = Reporter::new()
        .add(&room)
        .add(&socket)
        .add(&thermo)
        .report();

    println!("\nReporter output:");
    print!("{report}");

    // ═══════════════════════════════════════════════════════════════════════
    // 4. Dynamic management demo (from Assignment 2)
    // ═══════════════════════════════════════════════════════════════════════
    println!("\n═══ Dynamic management ═══");

    home.add_room("bathroom", Room::new("Bathroom"));
    println!("Added 'bathroom'.");

    if let Some(bathroom) = home.get_room_mut("bathroom") {
        bathroom.add_device("light", Socket::new("Bathroom light", 60.0));
        bathroom.subscribe(|d: &SmartDevice| {
            println!(
                "  Bathroom got new device: {}",
                d.report().lines().next().unwrap_or("")
            );
        });
        // This subscriber will fire for the next add:
        bathroom.add_device("fan", Socket::new("Exhaust fan", 50.0));
    }

    print_report("FINAL HOME STATE", &home);

    // Error handling demo
    println!("\n═══ Error handling ═══");
    match home.get_device("bedroom", "heater") {
        Ok(d) => println!("Found: {}", d.report()),
        Err(e) => println!("Error: {e}"),
    }
    match home.get_device("nonexistent", "lamp") {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected error: {e}"),
    }

    Ok(())
}

// ── Subscriber implementation ──────────────────────────────────────────────

/// A simple subscriber that prints every added device.
struct PrintSubscriber;

impl Subscriber for PrintSubscriber {
    fn on_event(&mut self, device: &SmartDevice) {
        println!(
            "  [PrintSubscriber] New device: {}",
            device.report().lines().next().unwrap_or("")
        );
    }
}
