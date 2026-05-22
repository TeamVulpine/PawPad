use hidapi::HidDevice;
use pawkit_crockford::Ulid;
use uuid::Uuid;

use crate::{gamepad::GamepadEvent, mapping::BakedGamepadMappings};

#[derive(Debug)]
pub struct UnknwonDriver {
    vendor: u16,
    product: u16,
}

impl UnknwonDriver {
    pub fn new(vendor: u16, product: u16) -> Self {
        println!(
            "(PawPad) HID: Unknown vendor / product! {} / {}",
            vendor, product
        );
        return Self { vendor, product };
    }

    pub fn init(&self, device: &HidDevice) {}

    pub fn handle_state(
        &self,
        _state: &[u8],
        _id: Ulid,
        _uuid: Uuid,
        _alternative_uuid: Uuid,
        _events: &mut Vec<GamepadEvent>,
        _mappings: &BakedGamepadMappings,
    ) {
    }
}
