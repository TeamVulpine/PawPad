use hidapi::HidDevice;
use pawkit_crockford::Ulid;
use uuid::Uuid;

use crate::{
    backend::hid::driver::{eightbitdo::EightBitDoDriver, unknown::UnknwonDriver},
    gamepad::GamepadEvent,
    mapping::BakedGamepadMappings,
};

mod eightbitdo;
mod unknown;

#[derive(Debug)]
pub enum HidDriver {
    EightBitDo(EightBitDoDriver),
    Unknown(UnknwonDriver),
}

impl HidDriver {
    pub fn from(vendor: u16, product: u16) -> Self {
        if let Some(driver) = EightBitDoDriver::from(vendor, product) {
            return Self::EightBitDo(driver);
        }

        return Self::Unknown(UnknwonDriver::new(vendor, product));
    }

    pub fn init(&self, device: &HidDevice) {
        match self {
            Self::EightBitDo(driver) => driver.init(device),
            Self::Unknown(driver) => driver.init(device),
        }
    }

    pub fn handle_state(
        &mut self,
        packet: &[u8],
        id: Ulid,
        uuid: Uuid,
        alternative_uuid: Uuid,
        events: &mut Vec<GamepadEvent>,
        mappings: &BakedGamepadMappings,
    ) {
        match self {
            Self::EightBitDo(driver) => {
                driver.handle_state(packet, id, uuid, alternative_uuid, events, mappings)
            }
            Self::Unknown(driver) => {
                driver.handle_state(packet, id, uuid, alternative_uuid, events, mappings)
            }
        }
    }
}
