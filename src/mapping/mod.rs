use std::collections::HashMap;

use uuid::Uuid;

use crate::{gamepad::{axis::GamepadAxis, button::GamepadButton}, mapping::hat::HatDescriptor};

pub mod hat;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeviceMappings {
    name: Box<str>,
    #[cfg_attr(feature = "serde", serde(default))]
    buttons: HashMap<u16, GamepadButton>,
    #[cfg_attr(feature = "serde", serde(default))]
    axes: HashMap<u16, GamepadAxis>,
    #[cfg_attr(feature = "serde", serde(default))]
    hats: HashMap<HatDescriptor, GamepadButton>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GamepadMappings {
    #[cfg_attr(feature = "serde", serde(default))]
    linux: HashMap<Uuid, DeviceMappings>,
    #[cfg_attr(feature = "serde", serde(default))]
    windows: HashMap<Uuid, DeviceMappings>,
    #[cfg_attr(feature = "serde", serde(default))]
    macos: HashMap<Uuid, DeviceMappings>,
}

impl GamepadMappings {
    fn get_platform(&self) -> &HashMap<Uuid, DeviceMappings> {
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

        for (id, mappings) in platform {
            for (index, button) in &mappings.buttons {
                buttons.insert((*id, *index), *button);
            }
            for (index, axis) in &mappings.axes {
                axes.insert((*id, *index), *axis);
            }
            for (descriptor, button) in &mappings.hats {
                hats.insert((*id, *descriptor), *button);
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
    buttons: HashMap<(Uuid, u16), GamepadButton>,
    axes: HashMap<(Uuid, u16), GamepadAxis>,
    hats: HashMap<(Uuid, HatDescriptor), GamepadButton>,
}

impl BakedGamepadMappings {
    pub fn get_button(&self, device_id: Uuid, index: u16) -> Option<GamepadButton> {
        return self.buttons.get(&(device_id, index)).cloned();
    }

    pub fn get_axis(&self, device_id: Uuid, index: u16) -> Option<GamepadAxis> {
        return self.axes.get(&(device_id, index)).cloned();
    }

    pub fn get_hat(&self, device_id: Uuid, descriptor: HatDescriptor) -> Option<GamepadButton> {
        return self.hats.get(&(device_id, descriptor)).cloned();
    }
}
