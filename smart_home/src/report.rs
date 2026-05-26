//! Report trait for smart home entities.

/// Trait for types that can generate a human-readable text report.
pub trait Report {
    /// Returns a formatted text report describing the current state.
    #[must_use]
    fn report(&self) -> String;
}
