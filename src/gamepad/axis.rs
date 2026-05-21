#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GamepadAxis {
    LeftX = 0,
    LeftY = 1,
    RightX = 2,
    RightY = 3,
    LeftTrigger = 4,
    RightTrigger = 5,
}

impl GamepadAxis {
    pub(crate) fn normalize(&self, value: f32) -> f32 {
        return match self {
            Self::LeftX | Self::LeftY | Self::RightX | Self::RightY => value * 2. - 1.,
            Self::LeftTrigger | Self::RightTrigger => value
        };
    }
}
