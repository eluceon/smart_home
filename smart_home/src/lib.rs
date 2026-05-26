//! "Smart Home" library.
//!
//! Provides building blocks for a smart home: devices (thermometers, sockets)
//! organised into rooms and a house.  All types implement the [`Report`] trait
//! so their state can be inspected at any level of the hierarchy.
//!
//! Devices can operate locally (in-process mock) or via the network:
//! - [`Socket::new_tcp`] connects to a running `socket_emulator` over TCP.
//! - [`Thermometer::new_udp`] listens for UDP datagrams from a `thermo_emulator`.
//!
//! # Patterns (Assignment 4)
//!
//! - [`HomeBuilder`] — typestate builder that prevents adding devices before
//!   the first room is created.
//! - [`Reporter`] — statically-typed heterogeneous list for composing reports.
//! - [`Subscriber`] — observer pattern: register callbacks that fire when a
//!   device is added to a [`Room`].
#![warn(missing_docs)]

pub mod builder;
pub mod devices;
pub mod error;
pub mod report;
pub mod reporter;
pub mod room;
pub mod smart_device;
pub mod smart_home;

pub use builder::HomeBuilder;
pub use devices::socket::{CMD_POWER, CMD_STATUS, CMD_TURN_OFF, CMD_TURN_ON};
pub use devices::{Socket, Thermometer};
pub use error::{NetworkError, SmartHomeError};
pub use report::Report;
pub use reporter::Reporter;
pub use room::{Room, Subscriber};
pub use smart_device::SmartDevice;
pub use smart_home::SmartHome;

/// Creates a [`Room`] from a list of `(key, device)` pairs.
///
/// Each device can be a [`Socket`] or a [`Thermometer`]; both are automatically
/// converted to [`SmartDevice`] via the `From` trait.
///
/// # Examples
///
/// ```
/// use smart_home::{room, Socket, Thermometer};
///
/// let r = room!(
///     "Living room",
///     "lamp"   => Socket::new("Desk lamp", 60.0),
///     "sensor" => Thermometer::new("Temp sensor", 22.5),
/// );
/// assert_eq!(r.name(), "Living room");
/// assert_eq!(r.device_count(), 2);
/// ```
#[macro_export]
macro_rules! room {
    // No devices — just create an empty room.
    ($name:expr $(,)?) => {
        $crate::Room::new($name)
    };
    // One or more (key => device) pairs.
    ($name:expr, $($key:expr => $device:expr),+ $(,)?) => {{
        let mut room = $crate::Room::new($name);
        $(
            room.add_device($key, $device);
        )+
        room
    }};
}
