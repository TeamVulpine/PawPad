use thiserror::Error;

use crate::backend::hid::{HidBackend, HidBackendError};

pub mod hid;

pub(crate) struct PawPadBackend {
    hid: HidBackend,
}

#[derive(Debug, Error)]
pub enum PawPadBackendError {
    #[error(transparent)]
    Hid(#[from] HidBackendError),
}

impl PawPadBackend {
    pub fn new() -> Result<Self, PawPadBackendError> {
        return Ok(Self {
            hid: HidBackend::new()?,
        });
    }

    pub fn update(&mut self) -> Result<(), PawPadBackendError> {
        self.hid.update()?;

        return Ok(());
    }
}
