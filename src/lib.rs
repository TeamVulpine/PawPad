use std::vec::Drain;

use thiserror::Error;

use crate::{
    backend::{PawPadBackend, PawPadBackendError},
    gamepad::GamepadEvent,
    mapping::{BakedGamepadMappings, GamepadMappings},
};

pub mod backend;
pub mod gamepad;
pub mod mapping;

#[derive(Debug, Error)]
#[error(transparent)]
pub struct PawPadError(#[from] PawPadBackendError);

pub struct PawPad {
    backend: PawPadBackend,
    mappings: BakedGamepadMappings,
}

impl PawPad {
    pub fn new(mappings: &GamepadMappings) -> Result<Self, PawPadError> {
        return Ok(Self {
            backend: PawPadBackend::new()?,
            mappings: mappings.bake(),
        });
    }

    pub fn poll_events<'a>(&'a mut self) -> Result<Drain<'a, GamepadEvent>, PawPadError> {
        return self.backend.poll_events(&self.mappings).map_err(Into::into);
    }
}
