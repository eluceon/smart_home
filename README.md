# Smart Home

A Rust library modelling a smart home: thermometers, sockets, rooms, and a
house. Supports local (in-process) and network (TCP/UDP) device backends.

## Features

- **Devices**: thermometer (local / UDP), socket (local / TCP)
- **Hierarchy**: `SmartDevice` → `Room` → `SmartHome`, all implement the
  `Report` trait
- **Error handling**: typed errors via `thiserror` — no panics in library code
- **Network emulators**: `socket_emulator` (TCP) and `thermo_emulator` (UDP)
  binaries
- **Design patterns**: typestate builder, static polymorphism reporter, observer
- **Tests**: 36 unit + 14 integration + 5 doctests; clippy- and fmt-clean

## Quick start

```bash
cargo build
cargo test
cargo run --example demo
```

## Examples

| Example | Description |
|---------|-------------|
| `cargo run --example demo` | Full API: room macro, dynamic management, error handling |
| `cargo run --example demo_network` | TCP sockets + UDP thermometers with emulators |
| `cargo run --example demo_patterns` | Builder, Reporter, Observer patterns |

### Network example (three terminals)

```bash
# Terminal 1 — TCP socket emulator
cargo run --bin socket_emulator -- 127.0.0.1:55001

# Terminal 2 — UDP thermometer emulator
cargo run --bin thermo_emulator

# Terminal 3 — smart home example
cargo run --example demo_network
```

## API overview

### Thermometer

```rust
use smart_home::Thermometer;

// Local (mock)
let thermo = Thermometer::new("Kitchen", 22.5);
assert_eq!(thermo.temperature().unwrap(), 22.5);

// UDP — receives datagrams from thermo_emulator
let thermo = Thermometer::new_udp("Kitchen", "127.0.0.1:8890")?;
// temperature() returns Err(NoDataReceived) until the first packet
```

### Socket

```rust
use smart_home::Socket;

// Local (mock)
let mut socket = Socket::new("Lamp", 60.0);
socket.turn_on()?;
assert!(socket.is_on()?);
assert_eq!(socket.power()?, 60.0);

// TCP — connects to socket_emulator
let socket = Socket::new_tcp("Lamp", "127.0.0.1:55001");
```

### SmartDevice, Room, SmartHome

```rust
use smart_home::{room, Room, SmartDevice, SmartHome};

// room! macro
let r = room!("Living room",
    "lamp"   => Socket::new("Lamp", 60.0),
    "sensor" => Thermometer::new("Sensor", 22.5),
);

// SmartDevice wraps any device
let device: SmartDevice = Socket::new("Lamp", 60.0).into();

// Home with dynamic management
let mut home = SmartHome::new("Apartment");
home.add_room("living", r);
home.add_room("bedroom", Room::new("Bedroom"));

// Error-typed device lookup
match home.get_device("living", "lamp") {
    Ok(d) => println!("{}", d.report()),
    Err(SmartHomeError::RoomNotFound(name)) => eprintln!("No room: {name}"),
    Err(SmartHomeError::DeviceNotFound(name)) => eprintln!("No device: {name}"),
    Err(e) => eprintln!("{e}"),
}
```

### Report trait

Every level of the hierarchy implements `Report`:

```rust
use smart_home::Report;

fn print_report(item: &impl Report) {
    println!("{}", item.report());
}

print_report(&device);
print_report(&room);
print_report(&home);
```

## Patterns (Assignment 4)

### HomeBuilder — typestate pattern

```rust
use smart_home::{HomeBuilder, Socket, Thermometer};

let home = HomeBuilder::new()
    .add_room("Living room")
    .add_device("lamp", Socket::default())
    .add_device("sensor", Thermometer::default())
    .add_room("Bedroom")
    .add_device("heater", Socket::default())
    .build();

// Compile-time safety:
// HomeBuilder::new().add_device(...) — does NOT compile
```

### Reporter — static polymorphism

```rust
use smart_home::Reporter;

let report = Reporter::new()
    .add(&room)
    .add(&socket)
    .add(&thermo)
    .report();
```

### Observer — dynamic polymorphism

```rust
use smart_home::Subscriber;

// Struct subscriber
room.subscribe(MySubscriber);

// Closure subscriber
room.subscribe(|device: &SmartDevice| {
    println!("Added: {}", device.report());
});
```

## CI checks

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test --locked
cargo doc --no-deps
```

## License

MIT
