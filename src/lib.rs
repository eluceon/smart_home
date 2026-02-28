//! "Smart Home" library.
//!
//! Provides building blocks for a smart home: devices (thermometers, sockets)
//! organised into rooms and a house.  All types implement the [`Report`] trait
//! so their state can be inspected at any level of the hierarchy.

pub mod devices;
pub mod error;
pub mod report;
pub mod room;
pub mod smart_device;
pub mod smart_home;

pub use devices::{Socket, Thermometer};
pub use error::SmartHomeError;
pub use report::Report;
pub use room::Room;
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
