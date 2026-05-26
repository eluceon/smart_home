# Smart Home

A Rust workspace modelling a smart home: thermometers, sockets, rooms, and a
house. Supports local (in-process) and network (TCP/UDP) device backends,
plus a C ABI socket library with static and dynamic linking.

## Workspace structure

| Crate | Description |
|-------|-------------|
| `smart_home` | Core library — devices, rooms, hierarchy, patterns |
| `socket-c` | C ABI socket library — `staticlib`, `cdylib`, `rlib` |
| `app-static` | Demo app statically linking `socket-c` |
| `app-dynamic` | Demo app dynamically loading `libsocket_c.so` via `libloading` |

## Quick start

```bash
cargo build --workspace
cargo test --workspace
cargo run --example demo
```

## Features

- **Devices**: thermometer (local / UDP), socket (local / TCP)
- **Hierarchy**: `SmartDevice` → `Room` → `SmartHome`, all implement `Report`
- **Error handling**: typed errors via `thiserror` — no panics in library code
- **Network emulators**: `socket_emulator` (TCP) and `thermo_emulator` (UDP)
- **Design patterns**: typestate builder, static reporter, observer
- **C ABI**: opaque-handle C API for smart socket with static and dynamic linking
- **CI**: tests, clippy, fmt, doc via GitHub Actions

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

let r = room!("Living room",
    "lamp"   => Socket::new("Lamp", 60.0),
    "sensor" => Thermometer::new("Sensor", 22.5),
);

let device: SmartDevice = Socket::new("Lamp", 60.0).into();

let mut home = SmartHome::new("Apartment");
home.add_room("living", r);

match home.get_device("living", "lamp") {
    Ok(d) => println!("{}", d.report()),
    Err(SmartHomeError::RoomNotFound(name)) => eprintln!("No room: {name}"),
    Err(SmartHomeError::DeviceNotFound(name)) => eprintln!("No device: {name}"),
    Err(e) => eprintln!("{e}"),
}
```

### Report trait

```rust
use smart_home::Report;

fn print_report(item: &impl Report) {
    println!("{}", item.report());
}
```

## Patterns

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

// HomeBuilder::new().add_device(...) — compile error, no room yet
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

room.subscribe(MySubscriber);
room.subscribe(|device: &SmartDevice| {
    println!("Added: {}", device.report());
});
```

## C ABI socket library

`socket-c` exposes an opaque-handle C API. Build artefacts:

| Artifact | Crate type |
|----------|------------|
| `libsocket_c.rlib` | Rust library |
| `libsocket_c.a` | Static C library |
| `libsocket_c.so` | Dynamic C library |

### Static linking

```bash
cargo run -p app-static
```

### Dynamic linking

```bash
cargo build -p socket-c
cargo run -p app-dynamic
# Or specify library path:
SOCKET_C_LIB=target/debug/libsocket_c.so cargo run -p app-dynamic
```

## CI

[![CI](https://github.com/eluceon/smart_home/actions/workflows/ci.yml/badge.svg)](https://github.com/eluceon/smart_home/actions/workflows/ci.yml)

On every push and PR: `cargo fmt --check`, `cargo clippy -- -D warnings`,
`cargo test --locked`, `cargo doc --no-deps`.

## License

MIT
