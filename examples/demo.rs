//! Example usage of the "Smart Home" library.
//!
//! Demonstrates:
//!  - Creating rooms with the `room!` macro.
//!  - Dynamic room and device management.
//!  - The `Report` trait through a generic helper function.
//!  - Error handling when a room or device is not found.

use smart_home::{room, Report, SmartDevice, SmartHome, Socket, Thermometer};

/// Prints a labelled report for any type that implements [`Report`].
fn print_report<R: Report>(label: &str, item: &R) {
    let sep = "-".repeat(40);
    println!("\n{}\n{}\n{}", sep, label, sep);
    print!("{}", item.report());
}

fn main() {
    // ── Build initial home with the `room!` macro ─────────────────────────

    let living_room = room!(
        "Living room",
        "sensor" => Thermometer::new("Temperature sensor", 22.5),
        "lamp"   => Socket::new("Desk lamp", 60.0),
        "ac"     => Socket::new("Air conditioner", 1500.0),
    );

    let bedroom = room!(
        "Bedroom",
        "sensor"        => Thermometer::new("Temperature sensor", 20.0),
        "ceiling_light" => Socket::new("Ceiling light", 100.0),
        "heater"        => Socket::new("Space heater", 2000.0),
    );

    let kitchen = room!(
        "Kitchen",
        "sensor" => Thermometer::new("Temperature sensor", 24.0),
        "kettle" => Socket::new("Kettle", 2500.0),
        "fridge" => Socket::new("Fridge", 800.0),
    );

    let mut home = SmartHome::new("My Smart Home");
    home.add_room("living_room", living_room);
    home.add_room("bedroom", bedroom);
    home.add_room("kitchen", kitchen);

    // ── Full home report via the Report trait ─────────────────────────────

    print_report("INITIAL HOME STATE", &home);

    // ── Dynamic device management ─────────────────────────────────────────

    println!("\n=== Dynamic device manipulation ===");

    if let Some(bedroom) = home.get_room_mut("bedroom") {
        bedroom.add_device("night_lamp", Socket::new("Night lamp", 10.0));
        println!("Added 'night_lamp' to bedroom.");

        let removed = bedroom.remove_device("heater");
        if removed.is_some() {
            println!("Removed 'heater' from bedroom.");
        }

        if let Some(socket) = bedroom
            .get_device_mut("ceiling_light")
            .and_then(|d| d.as_socket_mut())
        {
            socket.turn_on();
            println!("Turned on 'ceiling_light' in bedroom.");
        }
    }

    // Report on a single room
    if let Some(bedroom) = home.get_room("bedroom") {
        print_report("BEDROOM AFTER CHANGES", bedroom);
    }

    // ── Dynamic room management ───────────────────────────────────────────

    println!("\n=== Dynamic room manipulation ===");

    let bathroom = room!(
        "Bathroom",
        "light"  => Socket::new("Bathroom light", 60.0),
        "sensor" => Thermometer::new("Humidity sensor", 25.0),
    );
    home.add_room("bathroom", bathroom);
    println!("Added 'bathroom'.");

    if home.remove_room("kitchen").is_some() {
        println!("Removed 'kitchen'.");
    }

    // ── Individual device reports ─────────────────────────────────────────

    println!("\n=== Individual device reports ===");

    let table_lamp: SmartDevice = Socket::new("Table lamp", 40.0).into();
    print_report("Table lamp (socket)", &table_lamp);

    let hallway_sensor: SmartDevice = Thermometer::new("Hallway sensor", 19.5).into();
    print_report("Hallway sensor (thermometer)", &hallway_sensor);

    // ── Error handling ────────────────────────────────────────────────────

    println!("\n=== Error handling ===");

    match home.get_device("living_room", "sensor") {
        Ok(device) => println!("Found device: {}", device.report()),
        Err(e) => println!("Unexpected error: {}", e),
    }

    match home.get_device("nonexistent_room", "sensor") {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected error (room): {}", e),
    }

    match home.get_device("living_room", "nonexistent_device") {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected error (device): {}", e),
    }

    // ── Final home report ─────────────────────────────────────────────────

    print_report("FINAL HOME STATE", &home);
}
