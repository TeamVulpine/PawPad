use std::time::SystemTime;

use bitflags::bitflags;
use hidapi::HidDevice;
use pawkit_crockford::Ulid;
use uuid::Uuid;

use crate::{
    gamepad::{
        GamepadEvent, GamepadEventKind, GamepadId, axis::GamepadAxis, button::GamepadButton,
    },
    mapping::{
        AxisMapping, BakedGamepadMappings,
        hat::{HatButton, HatDescriptor, HatIndex},
    },
};

#[derive(Debug)]
#[allow(unused)]
pub struct EightBitDoDriver {
    product: Product,
    last_button: u32,
    last_hat: HatMask,
    last_axes: [u8; 6],
}

#[derive(Debug)]
enum Transport {
    Usb,
    Bluetooth,
}

#[derive(Debug)]
#[allow(unused)]
enum Product {
    Sn30Pro(Transport),
    Pro2(Transport),
    Pro3,
    Ultimate2,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeviceHat {
    Up = 0,
    RightUp = 1,
    Right = 2,
    RightDown = 3,
    Down = 4,
    LeftDown = 5,
    Left = 6,
    LeftUp = 7,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct HatMask: u8 {
        const UP = 1 << 0;
        const RIGHT = 1 << 1;
        const DOWN = 1 << 2;
        const LEFT = 1 << 3;
    }
}

impl DeviceHat {
    pub fn from_device(index: u8) -> Option<Self> {
        return match index {
            0 => Some(Self::Up),
            1 => Some(Self::RightUp),
            2 => Some(Self::Right),
            3 => Some(Self::RightDown),
            4 => Some(Self::Down),
            5 => Some(Self::LeftDown),
            6 => Some(Self::Left),
            7 => Some(Self::LeftUp),
            _ => None,
        };
    }

    pub fn to_mask(self) -> HatMask {
        return match self {
            Self::Up => HatMask::UP,
            Self::RightUp => HatMask::UP | HatMask::RIGHT,
            Self::Right => HatMask::RIGHT,
            Self::RightDown => HatMask::RIGHT | HatMask::DOWN,
            Self::Down => HatMask::DOWN,
            Self::LeftDown => HatMask::DOWN | HatMask::LEFT,
            Self::Left => HatMask::LEFT,
            Self::LeftUp => HatMask::LEFT | HatMask::UP,
        };
    }
}

impl HatMask {
    pub fn iter_buttons(self) -> impl Iterator<Item = (HatMask, HatButton)> {
        return [
            (HatMask::UP, HatButton::One),
            (HatMask::RIGHT, HatButton::Two),
            (HatMask::DOWN, HatButton::Four),
            (HatMask::LEFT, HatButton::Eight),
        ]
        .into_iter()
        .filter(move |(mask, _)| self.contains(*mask));
    }
}

impl EightBitDoDriver {
    const VENDOR: u16 = 0x2dc8;

    const SN30_PRO: u16 = 0x6001;
    const SN30_PRO_BT: u16 = 0x6101;
    const PRO_2: u16 = 0x6003;
    const PRO_2_BT: u16 = 0x6006;
    const PRO_3: u16 = 0x6009;
    const ULTIMATE2: u16 = 0x6012;

    pub fn from(vendor: u16, product: u16) -> Option<Self> {
        if vendor != Self::VENDOR {
            return None;
        }

        let product = match product {
            Self::SN30_PRO => Product::Pro2(Transport::Usb),
            Self::SN30_PRO_BT => Product::Pro2(Transport::Bluetooth),
            Self::PRO_2 => Product::Pro2(Transport::Usb),
            Self::PRO_2_BT => Product::Pro2(Transport::Bluetooth),
            Self::PRO_3 => Product::Pro3,
            Self::ULTIMATE2 => Product::Ultimate2,
            _ => Product::Unknown,
        };

        return Some(Self {
            product,
            last_button: 0,
            last_hat: HatMask::empty(),
            last_axes: [127, 127, 127, 127, 0, 0],
        });
    }

    pub fn init(&self, _device: &HidDevice) {}

    pub fn handle_state(
        &mut self,
        state: &[u8],
        id: Ulid,
        uuid: Uuid,
        alternative_uuid: Uuid,
        events: &mut Vec<GamepadEvent>,
        mappings: &BakedGamepadMappings,
    ) {
        let timestamp = SystemTime::now();

        let hat = DeviceHat::from_device(state[1])
            .map(DeviceHat::to_mask)
            .unwrap_or(HatMask::empty());

        if hat != self.last_hat {
            let changed = hat.symmetric_difference(self.last_hat);

            for (index, button) in changed.iter_buttons() {
                let descriptor = HatDescriptor(HatIndex::Zero, button);
                let button = mappings
                    .get_hat(uuid, alternative_uuid, descriptor)
                    .unwrap_or_else(|| descriptor.guess_button());

                events.push(GamepadEvent {
                    id: GamepadId(id),
                    timestamp,
                    kind: GamepadEventKind::ButtonChanged(button, hat.contains(index)),
                });
            }

            self.last_hat = hat;
        }

        for i in 0..6 {
            let byte = state[i + 2];

            if self.last_axes[i] == byte {
                continue;
            }

            self.last_axes[i] = byte;

            let value = byte as f32 / u8::MAX as f32;

            if let Some(axis) = mappings
                .get_axis(uuid, alternative_uuid, i as u16)
                .or_else(|| guess_axis(i as u16))
            {
                events.push(GamepadEvent {
                    id: GamepadId(id),
                    timestamp,
                    kind: GamepadEventKind::AxisMoved(axis.axis, axis.normalize(value)),
                });
            }
        }

        let mut button_mask = state[8] as u32;

        button_mask |= (state[9] as u32) << 8;

        if let Some(state_10) = state.get(10) {
            button_mask |= (*state_10 as u32) << 16;
        }

        if button_mask != self.last_button {
            let diff = button_mask ^ self.last_button;

            for bit in 0..32 {
                let index = 1 << bit;

                let changed = diff & index != 0;

                if !changed {
                    continue;
                }

                let new_value = button_mask & index != 0;

                if let Some(button) = mappings
                    .get_button(uuid, alternative_uuid, bit)
                    .or_else(|| guess_button(bit))
                {
                    events.push(GamepadEvent {
                        id: GamepadId(id),
                        timestamp,
                        kind: GamepadEventKind::ButtonChanged(button, new_value),
                    });
                }
            }

            self.last_button = button_mask;
        }
    }
}

fn guess_axis(scancode: u16) -> Option<AxisMapping> {
    return match scancode {
        0 => Some(AxisMapping::of_axis(GamepadAxis::LeftX)),
        1 => Some(AxisMapping::of_axis(GamepadAxis::LeftY)),
        2 => Some(AxisMapping::of_axis(GamepadAxis::RightX)),
        3 => Some(AxisMapping::of_axis(GamepadAxis::RightY)),
        4 => Some(AxisMapping::of_axis(GamepadAxis::RightTrigger)),
        5 => Some(AxisMapping::of_axis(GamepadAxis::LeftTrigger)),
        _ => None,
    };
}

fn guess_button(scancode: u16) -> Option<GamepadButton> {
    return match scancode {
        0 => Some(GamepadButton::East),
        1 => Some(GamepadButton::South),
        2 => Some(GamepadButton::RightPaddle2),
        3 => Some(GamepadButton::North),
        4 => Some(GamepadButton::West),
        5 => Some(GamepadButton::LeftPaddle2),
        6 => Some(GamepadButton::LeftShoulder),
        7 => Some(GamepadButton::RightShoulder),
        10 => Some(GamepadButton::Back),
        11 => Some(GamepadButton::Start),
        12 => Some(GamepadButton::Guide),
        13 => Some(GamepadButton::LeftStick),
        14 => Some(GamepadButton::RightStick),
        16 => Some(GamepadButton::LeftPaddle1),
        17 => Some(GamepadButton::RightPaddle1),
        _ => None,
    };
}
