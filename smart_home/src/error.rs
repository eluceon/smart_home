//! Error types for the smart home library.

use thiserror::Error;

/// Errors that arise from network I/O, protocol violations, or synchronisation
/// failures.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum NetworkError {
    /// An underlying I/O error (connection refused, broken pipe, …).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The server returned an unexpected response.
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// No temperature data has been received yet (UDP stream not started).
    #[error("No data received yet")]
    NoDataReceived,

    /// Internal synchronisation lock is poisoned — a background thread panicked.
    #[error("Synchronisation error: {0}")]
    SyncPoison(String),
}

/// Errors that can occur when accessing rooms or devices in a smart home.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SmartHomeError {
    /// The requested room was not found.
    #[error("Room '{0}' not found")]
    RoomNotFound(String),

    /// The requested device was not found.
    #[error("Device '{0}' not found")]
    DeviceNotFound(String),

    /// A network-level error occurred while communicating with a device.
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
}
