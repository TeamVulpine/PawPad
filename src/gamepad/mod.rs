use std::time::SystemTime;

use pawkit_crockford::Ulid;

use crate::{
    gamepad::{
        axis::GamepadAxis, button::GamepadButton, pointer::GamepadPointer, sensor::GamepadSensor,
    },
    mapping::{Transport, device_id::DeviceId},
};

pub mod axis;
pub mod button;
pub mod pointer;
pub mod sensor;

/// An ID for a gamepad.
/// It is randomly generated when a gamepad is connected, per the ULID spec.
/// See https://github.com/ulid/spec
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GamepadId(pub(crate) Ulid);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GamepadEventKind {
    Connected(DeviceId, Transport),
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
    pub timestamp: SystemTime,
    pub kind: GamepadEventKind,
}
