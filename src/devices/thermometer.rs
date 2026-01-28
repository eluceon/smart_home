//! Smart thermometer.

/// Represents a smart thermometer.
#[derive(Debug, Clone)]
pub struct Thermometer {
    name: String,
    current_temperature: f32,
}

impl Thermometer {
    /// Creates a new smart thermometer.
    ///
    /// # Arguments
    ///
    /// * `name` - Thermometer name
    /// * `current_temperature` - Current temperature in Celsius
    ///
    /// # Examples
    ///
    /// ```
    /// use smart_home::Thermometer;
    ///
    /// let thermometer = Thermometer::new("Living room".to_string(), 22.5);
    /// assert_eq!(thermometer.temperature(), 22.5);
    /// ```
    pub fn new(name: String, current_temperature: f32) -> Self {
        Self {
            name,
            current_temperature,
        }
    }

    /// Returns the current temperature.
    pub fn temperature(&self) -> f32 {
        self.current_temperature
    }

    /// Returns the thermometer name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Updates the current temperature.
    pub fn set_temperature(&mut self, temperature: f32) {
        self.current_temperature = temperature;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thermometer_creation() {
        let thermometer = Thermometer::new("Living room thermometer".to_string(), 20.0);
        assert_eq!(thermometer.temperature(), 20.0);
        assert_eq!(thermometer.name(), "Living room thermometer");
    }

    #[test]
    fn test_thermometer_update_temperature() {
        let mut thermometer = Thermometer::new("Test thermometer".to_string(), 18.0);
        thermometer.set_temperature(25.5);
        assert_eq!(thermometer.temperature(), 25.5);
    }
}
