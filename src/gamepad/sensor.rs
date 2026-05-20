#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GamepadSensor {
    /// Samples in radians/s
    Gyroscope = 0,
    /// Samples in m/s/s
    Accelerometer = 1,
}
