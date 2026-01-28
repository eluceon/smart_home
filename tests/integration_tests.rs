use smart_home::{Room, SmartDevice, SmartHome, Socket, Thermometer};

#[test]
fn smart_home_report_and_toggle_socket() {
    let living_room_devices = vec![
        SmartDevice::Thermometer(Thermometer::new("Sensor".to_string(), 22.0)),
        SmartDevice::Socket(Socket::new("Lamp".to_string(), 60.0)),
    ];

    let bedroom_devices = vec![
        SmartDevice::Thermometer(Thermometer::new("Sensor".to_string(), 20.0)),
        SmartDevice::Socket(Socket::new("Space heater".to_string(), 2000.0)),
    ];

    let rooms = vec![
        Room::new("Living room".to_string(), living_room_devices),
        Room::new("Bedroom".to_string(), bedroom_devices),
    ];

    let mut home = SmartHome::new("Home".to_string(), rooms);

    // Initial report
    home.print_full_report();

    // Turn the bedroom socket on and off
    let socket = home
        .get_room_mut(1)
        .get_device_mut(1)
        .as_socket_mut()
        .expect("expected a socket");
    socket.turn_on();
    assert!(socket.is_on());
    assert_eq!(socket.power(), 2000.0);

    socket.turn_off();
    assert!(!socket.is_on());
    assert_eq!(socket.power(), 0.0);

    // Final report
    home.print_full_report();
}
