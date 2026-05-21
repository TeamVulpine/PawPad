use std::str::FromStr;

use pawpad::{
    gamepad::{
        axis::GamepadAxis,
        button::{self, GamepadButton},
    },
    mapping::{
        AxisMapping, DeviceMappings, GamepadMappings,
        hat::{HatButton, HatDescriptor, HatIndex},
    },
};
use uuid::Uuid;

const DB: &str = include_str!("../SDL_GameControllerDB/gamecontrollerdb.txt");

fn button(name: &str) -> Option<GamepadButton> {
    return match name {
        "a" => Some(GamepadButton::South),
        "b" => Some(GamepadButton::East),
        "x" => Some(GamepadButton::West),
        "y" => Some(GamepadButton::North),

        "back" => Some(GamepadButton::Back),
        "start" => Some(GamepadButton::Start),
        "guide" => Some(GamepadButton::Guide),

        "leftstick" => Some(GamepadButton::LeftStick),
        "rightstick" => Some(GamepadButton::RightStick),

        "leftshoulder" => Some(GamepadButton::LeftShoulder),
        "rightshoulder" => Some(GamepadButton::RightShoulder),

        "dpup" => Some(GamepadButton::DPadUp),
        "dpdown" => Some(GamepadButton::DPadDown),
        "dpleft" => Some(GamepadButton::DPadLeft),
        "dpright" => Some(GamepadButton::DPadRight),

        "paddle1" => Some(GamepadButton::RightPaddle1),
        "paddle2" => Some(GamepadButton::LeftPaddle1),
        "paddle3" => Some(GamepadButton::RightPaddle2),
        "paddle4" => Some(GamepadButton::LeftPaddle2),

        "misc1" => Some(GamepadButton::Misc1),
        "misc2" => Some(GamepadButton::Misc2),
        "misc3" => Some(GamepadButton::Misc3),
        "misc4" => Some(GamepadButton::Misc4),
        "misc5" => Some(GamepadButton::Misc5),
        "misc6" => Some(GamepadButton::Misc6),

        _ => None,
    };
}

fn axis(name: &str) -> Option<GamepadAxis> {
    return match name {
        "leftx" => Some(GamepadAxis::LeftX),
        "lefty" => Some(GamepadAxis::LeftY),
        "lefttrigger" => Some(GamepadAxis::LeftTrigger),
        "rightx" => Some(GamepadAxis::RightX),
        "righty" => Some(GamepadAxis::RightY),
        "righttrigger" => Some(GamepadAxis::RightTrigger),

        _ => return None,
    };
}

fn main() {
    let mut mappings = GamepadMappings::default();

    for mapping in DB
        .lines()
        .filter(|it| !it.is_empty() && !it.starts_with("#"))
    {
        let strings = mapping
            .split(",")
            .filter(|it| !it.is_empty())
            .collect::<Box<[&str]>>();

        let uuid = strings[0];
        let platform = strings[strings.len() - 1];

        let name = strings[1];

        let platform_name = &platform[9..];

        let platform = match platform_name {
            "Windows" => &mut mappings.windows,
            "Linux" => &mut mappings.linux,
            "Mac OS X" => &mut mappings.macos,
            "iOS" => &mut mappings.ios,
            "Android" => &mut mappings.android,
            _ => continue,
        };

        let Ok(uuid) = Uuid::from_str(uuid) else {
            continue;
        };

        let bindings = &strings[2..strings.len() - 1];

        let mut device = DeviceMappings::default();

        device.name = name.into();

        for binding in bindings {
            let (name, value) = binding.split_once(':').unwrap();

            if value.starts_with("b") {
                let value: u16 = value[1..].parse().unwrap();

                let Some(button) = button(name) else {
                    continue;
                };

                device.buttons.insert(value, button);
            }

            if value.starts_with("a") {
                let invert = value.ends_with('~');

                let end = if invert { value.len() - 1 } else { value.len() };

                let value: u16 = value[1..end].parse().unwrap();

                let Some(axis) = axis(name) else {
                    continue;
                };

                device.axes.insert(value, AxisMapping { axis, invert });
            }

            if value.starts_with("h") {
                let bytes = value[1..].as_bytes();

                if bytes.len() != 3 || bytes[1] != b'.' {
                    continue;
                }

                let Some(hat_index) = HatIndex::from_char(bytes[0]) else {
                    continue;
                };

                let Some(hat_button) = HatButton::from_char(bytes[2]) else {
                    continue;
                };

                let Some(button) = button(name) else {
                    continue;
                };

                device
                    .hats
                    .insert(HatDescriptor(hat_index, hat_button), button);
            }
        }

        platform.insert(uuid, device);
    }

    println!("{}", serde_json::to_string_pretty(&mappings).unwrap());
}
