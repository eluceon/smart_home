//! Error types for the smart home library.

use std::fmt;

/// Errors that can occur when accessing rooms or devices in a smart home.
#[derive(Debug)]
pub enum SmartHomeError {
    /// The requested room was not found.
    RoomNotFound(String),
    /// The requested device was not found.
    DeviceNotFound(String),
}

impl fmt::Display for SmartHomeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SmartHomeError::RoomNotFound(name) => write!(f, "Room '{}' not found", name),
            SmartHomeError::DeviceNotFound(name) => write!(f, "Device '{}' not found", name),
        }
    }
}

impl std::error::Error for SmartHomeError {}
