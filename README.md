# Smart Home (Rust library)

A compact Rust library that models a smart home with devices (thermometers and
sockets), rooms, and a house structure. The package also includes a runnable
example and tests.

## Features

- Modular structure: devices, rooms, home
- Type-safe device model via `enum`
- Public API docs with examples
- Unit and integration tests
- Clippy- and rustfmt-friendly code

## Usage

### Build

```bash
cargo build
```

### Run the example

```bash
cargo run --example demo
```

## Library API

### Thermometer

```rust
let mut therm = Thermometer::new("Living room".to_string(), 22.5);
let temp = therm.temperature();      // 22.5
therm.set_temperature(25.0);         // Update temperature
```

### Socket

```rust
let mut socket = Socket::new("Lamp".to_string(), 100.0);
socket.turn_on();
socket.turn_off();
let power = socket.power();
let status = socket.is_on();
```

### SmartDevice

```rust
let device = SmartDevice::Thermometer(therm);
// or
let device = SmartDevice::Socket(socket);

println!("{}", device);
```

### Room

```rust
let devices = vec![...];
let mut room = Room::new("Bedroom".to_string(), devices);

let device = room.get_device(0);
let device = room.get_device_mut(0);
room.print_report();
```

### SmartHome

```rust
let rooms = vec![...];
let mut house = SmartHome::new("My home".to_string(), rooms);

let room = house.get_room(0);
let room = house.get_room_mut(0);
house.print_full_report();
```

## Notes

This project is created for educational purposes.
