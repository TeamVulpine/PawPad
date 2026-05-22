use std::{
    collections::{HashMap, HashSet},
    ffi::CStr,
    io,
    ops::Deref,
    path::PathBuf,
    time::SystemTime,
};

use hidapi::{BusType, HidApi, HidDevice, HidError};
use pawkit_crockford::Ulid;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    backend::{
        guid::{alternative_guid, get_guid},
        hid::driver::HidDriver,
    },
    gamepad::{GamepadEvent, GamepadEventKind, GamepadId},
    mapping::BakedGamepadMappings,
};

mod driver;

struct Device {
    device: HidDevice,
    driver: HidDriver,
    uuid: Uuid,
    alternative_uuid: Uuid,
}

pub(super) struct HidBackend {
    hid_api: HidApi,
    devices: HashMap<Ulid, Device>,
    device_paths: HashSet<PathBuf>,
}

#[derive(Debug, Error)]
pub enum HidBackendError {
    #[error(transparent)]
    Hid(#[from] HidError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

fn cstr_to_pathbuf(cstr: &CStr) -> PathBuf {
    let str = cstr.to_string_lossy();
    return PathBuf::from(str.deref());
}

impl HidBackend {
    const USAGE_PAGE_GENERIC_DESKTOP: u16 = 1;
    const USAGE_JOYSTICK: u16 = 4;
    const USAGE_GAMEPAD: u16 = 5;

    pub fn new() -> Result<Self, HidBackendError> {
        return Ok(Self {
            hid_api: HidApi::new()?,
            device_paths: HashSet::new(),
            devices: HashMap::new(),
        });
    }

    pub fn update(
        &mut self,
        events: &mut Vec<GamepadEvent>,
        mappings: &BakedGamepadMappings,
    ) -> Result<(), HidBackendError> {
        self.hid_api.refresh_devices()?;

        let devices = self.hid_api.device_list().filter(|it| {
            it.usage_page() == Self::USAGE_PAGE_GENERIC_DESKTOP
                && (it.usage() == Self::USAGE_GAMEPAD || it.usage() == Self::USAGE_JOYSTICK)
        });

        for info in devices {
            let path = cstr_to_pathbuf(info.path());

            if self.device_paths.contains(&path) {
                continue;
            }

            let Ok(device) = info.open_device(&self.hid_api) else {
                continue;
            };

            device.set_blocking_mode(false)?;

            let driver = HidDriver::from(info.vendor_id(), info.product_id());

            let bus_type = match info.bus_type() {
                BusType::Unknown => 0x00,
                BusType::Usb => 0x03,
                BusType::Bluetooth => 0x05,
                BusType::I2c => 0x18,
                BusType::Spi => 0x1c,
            };

            let uuid = get_guid(
                bus_type,
                info.vendor_id(),
                info.product_id(),
                info.release_number(),
                info.manufacturer_string(),
                info.product_string(),
                b'h',
                0,
            );

            let alternative_uuid = alternative_guid(uuid);

            driver.init(&device);

            let device = Device {
                device,
                driver,
                uuid,
                alternative_uuid,
            };

            let id = Ulid::new();

            self.devices.insert(id, device);
            self.device_paths.insert(path);

            events.push(GamepadEvent {
                id: GamepadId(id),
                timestamp: SystemTime::now(),
                kind: GamepadEventKind::Connected(uuid),
            });
        }

        for (id, device) in &mut self.devices {
            let mut buf = [0u8; 64];
            let mut last_read = 0;

            loop {
                let read = device.device.read(&mut buf)?;

                if read == 0 {
                    break;
                }

                last_read = read;
            }

            if last_read == 0 {
                continue;
            }

            device.driver.handle_state(
                &buf[..last_read],
                *id,
                device.uuid,
                device.alternative_uuid,
                events,
                mappings,
            );
        }

        return Ok(());
    }
}
