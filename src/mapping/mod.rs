use std::collections::HashMap;

use crate::{
    gamepad::{axis::GamepadAxis, button::GamepadButton},
    mapping::device_id::DeviceId,
};

pub mod device_id;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeviceMappings {
    name: Box<str>,
    #[cfg_attr(feature = "serde", serde(default))]
    buttons: HashMap<u16, GamepadButton>,
    #[cfg_attr(feature = "serde", serde(default))]
    axes: HashMap<u16, GamepadAxis>,
    #[cfg_attr(feature = "serde", serde(default))]
    hat: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Transport {
    Bluetooth,
    Usb,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlatformMappings {
    #[cfg_attr(feature = "serde", serde(default))]
    bluetooth: HashMap<DeviceId, DeviceMappings>,

    #[cfg_attr(feature = "serde", serde(default))]
    usb: HashMap<DeviceId, DeviceMappings>,
}

impl PlatformMappings {
    pub fn get_button(
        &self,
        transport: Transport,
        device_id: &DeviceId,
        index: &u16,
    ) -> Option<GamepadButton> {
        match transport {
            Transport::Bluetooth => {
                return self.bluetooth.get(device_id)?.buttons.get(index).cloned();
            }

            Transport::Usb => {
                return self.usb.get(device_id)?.buttons.get(index).cloned();
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GamepadMappings {
    #[cfg_attr(feature = "serde", serde(default))]
    linux: PlatformMappings,
    #[cfg_attr(feature = "serde", serde(default))]
    windows: PlatformMappings,
    #[cfg_attr(feature = "serde", serde(default))]
    macos: PlatformMappings,
}

impl GamepadMappings {
    fn get_platform(&self) -> &PlatformMappings {
        #[cfg(target_os = "linux")]
        {
            return &self.linux;
        }
        #[cfg(target_os = "windows")]
        {
            return &self.windows;
        }
        #[cfg(target_os = "macos")]
        {
            return &self.macos;
        }
    }

    pub(crate) fn bake(&self) -> BakedGamepadMappings {
        let platform = self.get_platform();

        let mut buttons = HashMap::new();
        let mut axes = HashMap::new();
        let mut hats = HashMap::new();

        for (id, mappings) in &platform.bluetooth {
            for (index, button) in &mappings.buttons {
                buttons.insert((Transport::Bluetooth, *id, *index), *button);
            }
            for (index, axis) in &mappings.axes {
                axes.insert((Transport::Bluetooth, *id, *index), *axis);
            }
            if let Some(hat) = mappings.hat {
                hats.insert((Transport::Bluetooth, *id), hat);
            }
        }

        for (id, mappings) in &platform.usb {
            for (index, button) in &mappings.buttons {
                buttons.insert((Transport::Usb, *id, *index), *button);
            }
            for (index, axis) in &mappings.axes {
                axes.insert((Transport::Usb, *id, *index), *axis);
            }
            if let Some(hat) = mappings.hat {
                hats.insert((Transport::Usb, *id), hat);
            }
        }

        return BakedGamepadMappings {
            buttons,
            axes,
            hats,
        };
    }
}

pub(crate) struct BakedGamepadMappings {
    buttons: HashMap<(Transport, DeviceId, u16), GamepadButton>,
    axes: HashMap<(Transport, DeviceId, u16), GamepadAxis>,
    hats: HashMap<(Transport, DeviceId), u8>,
}

impl BakedGamepadMappings {
    pub fn get_button(
        &self,
        transport: Transport,
        device_id: DeviceId,
        index: u16,
    ) -> Option<GamepadButton> {
        return self.buttons.get(&(transport, device_id, index)).cloned();
    }

    pub fn get_axis(
        &self,
        transport: Transport,
        device_id: DeviceId,
        index: u16,
    ) -> Option<GamepadAxis> {
        return self.axes.get(&(transport, device_id, index)).cloned();
    }

    pub fn get_hat(&self, transport: Transport, device_id: DeviceId) -> Option<u8> {
        return self.hats.get(&(transport, device_id)).cloned();
    }
}
