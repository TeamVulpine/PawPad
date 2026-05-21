use std::{
    collections::{HashMap, HashSet},
    ffi::{CStr, OsStr},
    io,
    ops::Deref,
    path::PathBuf,
    time::SystemTime,
};

use hidapi::{BusType, HidApi, HidDevice, HidError, MAX_REPORT_DESCRIPTOR_SIZE};
use hidreport::ReportDescriptor;
use pawkit_crockford::Ulid;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    backend::guid::get_guid,
    gamepad::{GamepadEvent, GamepadEventKind, GamepadId},
};

struct Device {
    device: HidDevice,
    report: ReportDescriptor,
    uuid: Uuid,
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
    #[error(transparent)]
    ReportDescriptor(#[from] hidreport::ParserError),
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

    pub fn update(&mut self, events: &mut Vec<GamepadEvent>) -> Result<(), HidBackendError> {
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

            let mut report_bytes = [0u8; MAX_REPORT_DESCRIPTOR_SIZE];

            let len = device.get_report_descriptor(&mut report_bytes)?;

            let report_bytes = &report_bytes[..len];

            let report_descriptor = ReportDescriptor::try_from(report_bytes)?;

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

            let device = Device {
                device: device,
                report: report_descriptor,
                uuid,
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

        return Ok(());
    }
}
