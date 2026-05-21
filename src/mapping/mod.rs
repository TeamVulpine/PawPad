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
    fn default_button(index: u16) -> Option<GamepadButton> {
        match index {
            0 => return Some(GamepadButton::South),
            1 => return Some(GamepadButton::East),
            3 => return Some(GamepadButton::West),
            4 => return Some(GamepadButton::North),
            6 => return Some(GamepadButton::LeftShoulder),
            7 => return Some(GamepadButton::RightShoulder),
            10 => return Some(GamepadButton::Back),
            11 => return Some(GamepadButton::Start),
            12 => return Some(GamepadButton::Guide),
            13 => return Some(GamepadButton::LeftStick),
            14 => return Some(GamepadButton::RightStick),
            _ => return None,
        }
    }

    fn default_axis(index: u16) -> Option<GamepadAxis> {
        match index {
            0 => return Some(GamepadAxis::LeftX),
            1 => return Some(GamepadAxis::LeftY),
            2 => return Some(GamepadAxis::RightX),
            3 => return Some(GamepadAxis::RightY),
            4 => return Some(GamepadAxis::RightTrigger),
            5 => return Some(GamepadAxis::LeftTrigger),
            _ => return None,
        }
    }

    fn default_hat(descriptor: HatDescriptor) -> Option<GamepadButton> {
        if descriptor.0 != HatIndex::Zero {
            return None;
        }

        match descriptor.1 {
            hat::HatButton::One => return Some(GamepadButton::DPadUp),
            hat::HatButton::Two => return Some(GamepadButton::DPadRight),
            hat::HatButton::Four => return Some(GamepadButton::DPadDown),
            hat::HatButton::Eight => return Some(GamepadButton::DPadLeft),
        }
    }

    pub fn get_button(&self, device_id: Uuid, index: u16) -> Option<GamepadButton> {
        return self
            .buttons
            .get(&(device_id, index))
            .cloned()
            .or_else(|| Self::default_button(index));
    }

    pub fn get_axis(&self, device_id: Uuid, index: u16) -> Option<AxisMapping> {
        return self.axes.get(&(device_id, index)).cloned().or_else(|| {
            Self::default_axis(index).map(|it| AxisMapping {
                axis: it,
                invert: false,
            })
        });
    }

    pub fn get_hat(&self, device_id: Uuid, descriptor: HatDescriptor) -> Option<GamepadButton> {
        return self
            .hats
            .get(&(device_id, descriptor))
            .cloned()
            .or_else(|| Self::default_hat(descriptor));
    }
}
