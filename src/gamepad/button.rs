#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GamepadButton {
    /// Bottom face button (A on Xbox)
    South = 0,
    /// Right face button (B on Xbox)
    East = 1,
    /// Left face button (X on Xbox)
    West = 2,
    /// Top face button (Y on Xbox)
    North = 3,

    Back = 4,
    Guide = 5,
    Start = 6,

    LeftStick = 7,
    RightStick = 8,

    LeftShoulder = 9,
    RightShoulder = 10,

    DPadUp = 11,
    DPadDown = 12,
    DPadLeft = 13,
    DPadRight = 14,

    RightPaddle1 = 15,
    RightPaddle2 = 16,
    LeftPaddle1 = 17,
    LeftPaddle2 = 18,

    Misc1 = 19,
    Misc2 = 20,
    Misc3 = 21,
    Misc4 = 22,
    Misc5 = 23,
    Misc6 = 24,
}
