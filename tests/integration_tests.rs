use smart_home::{room, Report, Room, SmartDevice, SmartHome, SmartHomeError, Socket, Thermometer};

// ── Helper ────────────────────────────────────────────────────────────────────

fn make_home() -> SmartHome {
    let mut home = SmartHome::new("Home");

    let mut living_room = Room::new("Living room");
    living_room.add_device("sensor", Thermometer::new("Sensor", 22.0));
    living_room.add_device("lamp", Socket::new("Lamp", 60.0));

    let mut bedroom = Room::new("Bedroom");
    bedroom.add_device("sensor", Thermometer::new("Sensor", 20.0));
    bedroom.add_device("heater", Socket::new("Space heater", 2000.0));

    home.add_room("living_room", living_room);
    home.add_room("bedroom", bedroom);
    home
}

// ── Report trait ──────────────────────────────────────────────────────────────

#[test]
fn test_report_contains_expected_data() {
    let home = make_home();
    let report = home.report();

    assert!(report.contains("Home"));
    assert!(report.contains("living_room"));
    assert!(report.contains("bedroom"));
}

// ── Toggle socket via mutable access ─────────────────────────────────────────

#[test]
fn smart_home_toggle_socket() {
    let mut home = make_home();

    let socket = home
        .get_room_mut("bedroom")
        .unwrap()
        .get_device_mut("heater")
        .unwrap()
        .as_socket_mut()
        .expect("expected a socket");

    socket.turn_on();
    assert!(socket.is_on());
    assert_eq!(socket.power(), 2000.0);

    socket.turn_off();
    assert!(!socket.is_on());
    assert_eq!(socket.power(), 0.0);

    let report = home.report();
    assert!(report.contains("Space heater"));
    assert!(report.contains("off"));
}

// ── Dynamic room management ───────────────────────────────────────────────────

#[test]
fn test_dynamic_room_management() {
    let mut home = SmartHome::new("Home");
    assert_eq!(home.room_count(), 0);

    home.add_room("kitchen", Room::new("Kitchen"));
    assert_eq!(home.room_count(), 1);
    assert!(home.get_room("kitchen").is_some());

    let removed = home.remove_room("kitchen");
    assert!(removed.is_some());
    assert_eq!(home.room_count(), 0);
    assert!(home.remove_room("kitchen").is_none());
}

// ── Dynamic device management ─────────────────────────────────────────────────

#[test]
fn test_dynamic_device_management() {
    let mut room = Room::new("Living room");
    assert_eq!(room.device_count(), 0);

    room.add_device("lamp", Socket::new("Lamp", 60.0));
    assert_eq!(room.device_count(), 1);
    assert!(room.get_device("lamp").is_some());

    assert!(room.remove_device("lamp").is_some());
    assert_eq!(room.device_count(), 0);
    assert!(room.remove_device("lamp").is_none());
}

// ── get_device error handling ─────────────────────────────────────────────────

#[test]
fn test_get_device_ok() {
    let home = make_home();
    assert!(home.get_device("living_room", "lamp").is_ok());
}

#[test]
fn test_get_device_room_not_found() {
    let home = make_home();
    let result = home.get_device("no_such_room", "lamp");
    assert!(matches!(result, Err(SmartHomeError::RoomNotFound(_))));
    assert!(result.unwrap_err().to_string().contains("no_such_room"));
}

#[test]
fn test_get_device_device_not_found() {
    let home = make_home();
    let result = home.get_device("living_room", "no_such_device");
    assert!(matches!(result, Err(SmartHomeError::DeviceNotFound(_))));
    assert!(result.unwrap_err().to_string().contains("no_such_device"));
}

// ── From<Socket> / From<Thermometer> for SmartDevice ─────────────────────────

#[test]
fn test_from_socket() {
    let device: SmartDevice = Socket::new("Lamp", 60.0).into();
    assert!(device.as_socket().is_some());
    assert!(device.as_thermometer().is_none());
}

#[test]
fn test_from_thermometer() {
    let device: SmartDevice = Thermometer::new("Sensor", 22.0).into();
    assert!(device.as_thermometer().is_some());
    assert!(device.as_socket().is_none());
}

// ── room! macro ───────────────────────────────────────────────────────────────

#[test]
fn test_room_macro_with_devices() {
    let room = room!(
        "Living room",
        "sensor" => Thermometer::new("Sensor", 22.0),
        "lamp"   => Socket::new("Lamp", 60.0),
    );
    assert_eq!(room.name(), "Living room");
    assert_eq!(room.device_count(), 2);
    assert!(room.get_device("sensor").is_some());
    assert!(room.get_device("lamp").is_some());
}

#[test]
fn test_room_macro_empty() {
    let room = room!("Empty");
    assert_eq!(room.name(), "Empty");
    assert_eq!(room.device_count(), 0);
}

// ── Report trait on all levels ────────────────────────────────────────────────

#[test]
fn test_report_trait_device() {
    let device: SmartDevice = Socket::new("Lamp", 60.0).into();
    let r = device.report();
    assert!(r.contains("Lamp"));
    assert!(r.contains("off"));
}

#[test]
fn test_report_trait_room() {
    let room = room!("Bedroom", "lamp" => Socket::new("Lamp", 60.0));
    let r = room.report();
    assert!(r.contains("Bedroom"));
    assert!(r.contains("lamp"));
}

#[test]
fn test_report_trait_home() {
    let home = make_home();
    let r = home.report();
    assert!(r.contains("Home"));
    assert!(r.contains("living_room"));
    assert!(r.contains("bedroom"));
}
