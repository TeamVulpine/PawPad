use std::time::Instant;

use crate::gamepad::{
    axis::GamepadAxis, button::GamepadButton, pointer::GamepadPointer, sensor::GamepadSensor,
};

pub mod axis;
pub mod button;
pub mod pointer;
pub mod sensor;

/// An ID for a gamepad.
/// The ID is unique per gamepad, but if a gamepad is disconnected,
/// the ID can be reused if a new one is connected.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GamepadId(u32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GamepadEventKind {
    Connected,
    Disconnected,
    ButtonChanged(GamepadButton, bool),
    AxisMoved(GamepadAxis, f32),
    SensorUpdated(GamepadSensor, [f32; 3]),
    PointerMoved(GamepadPointer, [f32; 2]),
    /// Event produced when the user touches the trackpad surface,
    /// This is different from clicking, because the user is not physically pressing it in
    PointerTouched(GamepadPointer, bool),
    /// Event produced when the user clicks the trackpad surface
    /// This is different from touching, because the user is physically pressing it in
    PointerClicked(GamepadPointer, bool),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GamepadEvent {
    pub id: GamepadId,
    pub timestamp: Instant,
    pub kind: GamepadEventKind,
}
