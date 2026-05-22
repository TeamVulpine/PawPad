use std::time::SystemTime;

use hidapi::HidDevice;
use pawkit_crockford::Ulid;
use uuid::Uuid;

use crate::{
    gamepad::{GamepadEvent, GamepadEventKind, GamepadId, button::GamepadButton},
    mapping::BakedGamepadMappings,
};

#[derive(Debug)]
pub struct EightBitDoDriver {
    product: Product,
    last_button: u32,
}

#[derive(Debug)]
enum Transport {
    Usb,
    Bluetooth,
}

#[derive(Debug)]
pub enum Product {
    Sn30Pro(Transport),
    Pro2(Transport),
    Pro3,
    Ultimate2,
    Unknown,
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
        });
    }

    pub fn init(&self, device: &HidDevice) {}

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

        // match state[1] {
        //     0 => println!("up"),
        //     1 => println!("right + up"),
        //     2 => println!("right"),
        //     3 => println!("right + down"),
        //     4 => println!("down"),
        //     5 => println!("left + down"),
        //     6 => println!("left"),
        //     7 => println!("left + up"),
        //     _ => println!("centered")
        // }
        // println!("{:}", state[2]);
        // println!("{:}", state[3]);
        // println!("{:}", state[4]);
        // println!("{:}", state[5]);
        // println!("{:}", state[6]);
        // println!("{:}", state[7]);

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

                if let Some(button) = mappings.get_button(uuid, alternative_uuid, bit).or_else(|| guess_button(bit)) {
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

fn guess_button(scancode: u16) -> Option<GamepadButton> {
    match scancode {
        0 => return Some(GamepadButton::East),
        1 => return Some(GamepadButton::South),
        2 => return Some(GamepadButton::RightPaddle2),
        3 => return Some(GamepadButton::North),
        4 => return Some(GamepadButton::West),
        5 => return Some(GamepadButton::LeftPaddle2),
        6 => return Some(GamepadButton::LeftShoulder),
        7 => return Some(GamepadButton::RightShoulder),
        10 => return Some(GamepadButton::Back),
        11 => return Some(GamepadButton::Start),
        12 => return Some(GamepadButton::Guide),
        13 => return Some(GamepadButton::LeftStick),
        14 => return Some(GamepadButton::RightStick),
        16 => return Some(GamepadButton::LeftPaddle1),
        17 => return Some(GamepadButton::RightPaddle1),
        _ => return None,
    }
}
