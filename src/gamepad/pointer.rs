#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GamepadPointer {
    /// Right touch pad on the Steam Deck / Steam Controller 2,
    /// the DualShock 4 / DualSense touchpad,
    /// or the right JoyCon 2 mouse
    Primary = 0,
    /// Left touch pad on the Steam Deck / Steam Controller 2,
    /// or the left JoyCon 2 mouse
    Secondary = 1,
}
