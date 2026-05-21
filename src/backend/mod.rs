use std::vec::Drain;

use thiserror::Error;

#[cfg(target_os = "linux")]
use crate::backend::evdev::{EvdevBackend, EvdevBackendError};
use crate::{
    backend::hid::{HidBackend, HidBackendError},
    gamepad::GamepadEvent,
    mapping::BakedGamepadMappings,
};

#[cfg(target_os = "linux")]
mod evdev;
mod guid;
mod hid;

pub(crate) struct PawPadBackend {
    hid: HidBackend,
    #[cfg(target_os = "linux")]
    evdev: EvdevBackend,

    events: Vec<GamepadEvent>,
}

#[derive(Debug, Error)]
pub(crate) enum PawPadBackendError {
    #[error(transparent)]
    Hid(#[from] HidBackendError),

    #[cfg(target_os = "linux")]
    #[error(transparent)]
    Evdev(#[from] EvdevBackendError),
}

impl PawPadBackend {
    pub fn new() -> Result<Self, PawPadBackendError> {
        return Ok(Self {
            hid: HidBackend::new()?,

            #[cfg(target_os = "linux")]
            evdev: EvdevBackend::new(),

            events: Vec::new(),
        });
    }

    fn update(&mut self, mappings: &BakedGamepadMappings) -> Result<(), PawPadBackendError> {
        self.hid.update(&mut self.events)?;

        #[cfg(target_os = "linux")]
        self.evdev.update(&mut self.events, mappings)?;

        return Ok(());
    }

    pub fn poll_events<'a>(
        &'a mut self,
        mappings: &BakedGamepadMappings,
    ) -> Result<Drain<'a, GamepadEvent>, PawPadBackendError> {
        self.update(mappings)?;

        return Ok(self.events.drain(..));
    }
}
