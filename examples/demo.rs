//! Example usage of the "Smart Home" library.

use smart_home::{Room, SmartDevice, SmartHome, Socket, Thermometer};

fn main() {
    // Devices for the living room
    let living_room_devices = vec![
        SmartDevice::Thermometer(Thermometer::new("Temperature sensor".to_string(), 22.5)),
        SmartDevice::Socket(Socket::new("Desk lamp".to_string(), 60.0)),
        SmartDevice::Socket(Socket::new("Air conditioner".to_string(), 1500.0)),
    ];

    // Devices for the bedroom
    let bedroom_devices = vec![
        SmartDevice::Thermometer(Thermometer::new("Temperature sensor".to_string(), 20.0)),
        SmartDevice::Socket(Socket::new("Ceiling light".to_string(), 100.0)),
        SmartDevice::Socket(Socket::new("Space heater".to_string(), 2000.0)),
    ];

    // Devices for the kitchen
    let kitchen_devices = vec![
        SmartDevice::Thermometer(Thermometer::new("Temperature sensor".to_string(), 24.0)),
        SmartDevice::Socket(Socket::new("Kettle".to_string(), 2500.0)),
        SmartDevice::Socket(Socket::new("Fridge".to_string(), 800.0)),
    ];

    // Rooms
    let rooms = vec![
        Room::new("Living room".to_string(), living_room_devices),
        Room::new("Bedroom".to_string(), bedroom_devices),
        Room::new("Kitchen".to_string(), kitchen_devices),
    ];

    // Smart home
    let mut smart_home = SmartHome::new("My smart home".to_string(), rooms);

    println!("\nINITIAL HOME STATE:");
    smart_home.print_full_report();

    println!("\nTurning some devices on...");

    // Turn on the living room lamp (room index 0, device index 1)
    if let Some(socket) = smart_home.get_room_mut(0).get_device_mut(1).as_socket_mut() {
        socket.turn_on();
        println!("Living room desk lamp is on");
    }

    // Turn on the bedroom ceiling light (room index 1, device index 1)
    if let Some(socket) = smart_home.get_room_mut(1).get_device_mut(1).as_socket_mut() {
        socket.turn_on();
        println!("Bedroom ceiling light is on");
    }

    // Turn on the kitchen kettle (room index 2, device index 1)
    if let Some(socket) = smart_home.get_room_mut(2).get_device_mut(1).as_socket_mut() {
        socket.turn_on();
        println!("Kitchen kettle is on");
    }

    println!("\nHOME STATE AFTER TURNING DEVICES ON:");
    smart_home.print_full_report();

    // Turn off the bedroom space heater (as required in the task)
    println!("\nTurning the bedroom space heater off...");
    if let Some(socket) = smart_home.get_room_mut(1).get_device_mut(2).as_socket_mut() {
        socket.turn_off();
        println!("Bedroom space heater is off");
    }

    println!("\nFINAL HOME STATE:");
    smart_home.print_full_report();
}
