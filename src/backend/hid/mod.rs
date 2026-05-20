use std::io;

use hidapi::{HidApi, HidError, MAX_REPORT_DESCRIPTOR_SIZE};
use thiserror::Error;

use crate::backend::hid::report_descriptor::HidReportDescriptor;

mod report_descriptor;

pub(super) struct HidBackend {
    hid_api: HidApi,
}

#[derive(Debug, Error)]
pub enum HidBackendError {
    #[error(transparent)]
    Hid(#[from] HidError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

impl HidBackend {
    const USAGE_PAGE_GENERIC_DESKTOP: u16 = 1;
    const USAGE_JOYSTICK: u16 = 4;
    const USAGE_GAMEPAD: u16 = 5;

    pub fn new() -> Result<Self, HidBackendError> {
        return Ok(Self {
            hid_api: HidApi::new()?,
        });
    }

    pub fn update(&mut self) -> Result<(), HidBackendError> {
        self.hid_api.refresh_devices()?;

        let devices = self.hid_api.device_list().filter(|it| {
            it.usage_page() == Self::USAGE_PAGE_GENERIC_DESKTOP
                && (it.usage() == Self::USAGE_GAMEPAD || it.usage() == Self::USAGE_JOYSTICK)
        });

        for info in devices {
            let Ok(device) = info.open_device(&self.hid_api) else {
                continue;
            };

            println!(
                "{:04x}:{:04x}, {:?}, {}, {}, {:?}, {:?}, {:?}",
                info.vendor_id(),
                info.product_id(),
                info.bus_type(),
                info.usage_page(),
                info.usage(),
                info.serial_number(),
                info.manufacturer_string(),
                info.product_string()
            );

            let report_descriptor = HidReportDescriptor::parse(&device)?;
        }

        return Ok(());
    }
}
