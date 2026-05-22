use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    gamepad::{axis::GamepadAxis, button::GamepadButton},
    mapping::hat::{HatDescriptor, HatIndex},
};

pub mod hat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AxisMapping {
    pub axis: GamepadAxis,
    pub invert: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeviceMappings {
    pub name: Box<str>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub buttons: HashMap<u16, GamepadButton>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub axes: HashMap<u16, AxisMapping>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub hats: HashMap<HatDescriptor, GamepadButton>,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GamepadMappings {
    #[cfg_attr(feature = "serde", serde(default))]
    pub linux: HashMap<Uuid, DeviceMappings>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub windows: HashMap<Uuid, DeviceMappings>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub macos: HashMap<Uuid, DeviceMappings>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub ios: HashMap<Uuid, DeviceMappings>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub android: HashMap<Uuid, DeviceMappings>,
}

impl AxisMapping {
    pub fn of_axis(axis: GamepadAxis) -> Self {
        return Self {
            axis,
            invert: false,
        }
    }

    pub fn normalize(&self, mut value: f32) -> f32 {
        if self.invert {
            value *= -1.;
            value += 1.;
        }

        return self.axis.normalize(value);
    }
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
        #[cfg(target_os = "ios")]
        {
            return &self.ios;
        }
        #[cfg(target_os = "android")]
        {
            return &self.android;
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
    axes: HashMap<(Uuid, u16), AxisMapping>,
    hats: HashMap<(Uuid, HatDescriptor), GamepadButton>,
}

impl BakedGamepadMappings {
    pub fn get_button(&self, uuid: Uuid, alternative_uuid: Uuid, index: u16) -> Option<GamepadButton> {
        return self.buttons.get(&(uuid, index)).or_else(|| self.buttons.get(&(alternative_uuid, index))).cloned();
    }

    pub fn get_axis(&self, uuid: Uuid, alternative_uuid: Uuid, index: u16) -> Option<AxisMapping> {
        return self.axes.get(&(uuid, index)).or_else(|| self.axes.get(&(alternative_uuid, index))).cloned();
    }

    pub fn get_hat(&self, uuid: Uuid, alternative_uuid: Uuid, descriptor: HatDescriptor) -> Option<GamepadButton> {
        return self.hats.get(&(uuid, descriptor)).or_else(|| self.hats.get(&(alternative_uuid, descriptor))).cloned();
    }
}
