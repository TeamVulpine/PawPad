use hidapi::HidDevice;
use pawkit_crockford::Ulid;
use uuid::Uuid;

use crate::{backend::hid::HidBackendError, gamepad::GamepadEvent, mapping::BakedGamepadMappings};

#[derive(Debug)]
pub struct UnknwonDriver {
    vendor: u16,
    product: u16,
}

impl UnknwonDriver {
    pub fn new(vendor: u16, product: u16) -> Self {
        return Self { vendor, product };
    }

    pub fn init(&self, device: &HidDevice) {}

    pub fn handle_state(
        &self,
        state: &[u8],
        id: Ulid,
        uuid: Uuid,
        events: &mut Vec<GamepadEvent>,
        mappings: &BakedGamepadMappings,
    ) {
    }
}
