use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io,
    ops::RangeInclusive,
    path::{Path, PathBuf},
    time::SystemTime,
};

use evdev::{AbsoluteAxisCode, Device, EventType, KeyCode};
use pawkit_crockford::Ulid;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    backend::guid::get_guid,
    gamepad::{GamepadEvent, GamepadEventKind, GamepadId, button::GamepadButton},
    mapping::{
        BakedGamepadMappings,
        hat::{HatButton, HatDescriptor, HatIndex},
    },
};

struct MappedDevice {
    device: Device,
    path: PathBuf,
    device_id: Uuid,
    keycode_mapping: HashMap<KeyCode, u16>,
    axis_mapping: HashMap<AbsoluteAxisCode, (u16, RangeInclusive<f32>)>,
    hatx: [Option<GamepadButton>; 4],
    haty: [Option<GamepadButton>; 4],
}

pub struct EvdevBackend {
    devices: HashMap<Ulid, MappedDevice>,
    device_paths: HashSet<PathBuf>,
}

#[derive(Debug, Error)]
pub enum EvdevBackendError {
    #[error(transparent)]
    Io(#[from] io::Error),
}

fn looks_like_gamepad(device: &Device) -> bool {
    let Some(keys) = device.supported_keys() else {
        return false;
    };

    let Some(abs) = device.supported_absolute_axes() else {
        return false;
    };

    let has_basic_buttons =
        keys.contains(evdev::KeyCode::BTN_SOUTH) || keys.contains(evdev::KeyCode::BTN_EAST);

    let has_analog_sticks = abs.contains(evdev::AbsoluteAxisCode::ABS_X)
        && abs.contains(evdev::AbsoluteAxisCode::ABS_Y);

    let has_dpad = (keys.contains(evdev::KeyCode::BTN_DPAD_UP)
        && keys.contains(evdev::KeyCode::BTN_DPAD_DOWN)
        && keys.contains(evdev::KeyCode::BTN_DPAD_LEFT)
        && keys.contains(evdev::KeyCode::BTN_DPAD_RIGHT))
        || (abs.contains(AbsoluteAxisCode::ABS_HAT0X) && abs.contains(AbsoluteAxisCode::ABS_HAT0Y));

    return has_basic_buttons && has_analog_sticks && has_dpad;
}

fn build_keycode_mapping(device: &Device) -> HashMap<KeyCode, u16> {
    let mut map = HashMap::new();

    for i in 0..u16::MAX {
        let Ok((keycode, _)) = device.get_scancode_by_index(i) else {
            break;
        };

        let key_code = KeyCode::new(keycode as u16);

        map.insert(key_code, i);
    }

    return map;
}

fn build_axis_mapping(
    device: &Device,
) -> Result<HashMap<AbsoluteAxisCode, (u16, RangeInclusive<f32>)>, io::Error> {
    let mut map = HashMap::new();

    for (i, (code, info)) in device.get_absinfo()?.enumerate() {
        map.insert(
            code,
            (i as u16, info.minimum() as f32..=info.maximum() as f32),
        );
    }

    return Ok(map);
}

fn guess_button(keycode: KeyCode) -> Option<GamepadButton> {
    match keycode {
        _ => {
            println!("(PawPad) EvdevBackend: Unexpected KeyCode: {:?}", keycode);
            return None;
        }
    }
}

impl EvdevBackend {
    const PATH: &str = "/dev/input";

    pub fn new() -> Self {
        return Self {
            device_paths: HashSet::new(),
            devices: HashMap::new(),
        };
    }

    pub fn update(
        &mut self,
        events: &mut Vec<GamepadEvent>,
        mappings: &BakedGamepadMappings,
    ) -> Result<(), EvdevBackendError> {
        for file in fs::read_dir(Self::PATH)? {
            let Ok(file) = file else {
                continue;
            };

            if self.device_paths.contains(&file.path()) {
                continue;
            }

            let path = Path::new("/sys/class/input")
                .join(file.file_name())
                .join("device/device/hidraw");

            if path.exists() {
                let Ok(read_dir) = fs::read_dir(path) else {
                    continue;
                };

                let mut skip_gamepad = false;

                for file in read_dir {
                    let Ok(file) = file else {
                        continue;
                    };

                    let path = Path::new("/dev").join(file.file_name());

                    match File::open(path) {
                        Ok(_) => {
                            skip_gamepad = true;
                        }
                        _ => {}
                    }
                }

                if skip_gamepad {
                    continue;
                }
            }

            let Ok(metadata) = file.metadata() else {
                continue;
            };

            if metadata.is_dir() {
                continue;
            }

            let Ok(device) = Device::open(file.path()) else {
                continue;
            };

            if !looks_like_gamepad(&device) {
                continue;
            }

            let Ok(_) = device.set_nonblocking(true) else {
                continue;
            };

            let input_id = device.input_id();

            let uuid = get_guid(
                input_id.bus_type().0,
                input_id.vendor(),
                input_id.product(),
                input_id.version(),
                None,
                device.name(),
                0,
                0,
            );

            let id = Ulid::new();

            let mapped = MappedDevice {
                keycode_mapping: build_keycode_mapping(&device),
                axis_mapping: build_axis_mapping(&device)?,
                device,
                device_id: uuid,
                path: file.path(),
                hatx: [None; 4],
                haty: [None; 4],
            };

            self.devices.insert(id, mapped);
            self.device_paths.insert(file.path());

            events.push(GamepadEvent {
                id: GamepadId(id),
                timestamp: SystemTime::now(),
                kind: GamepadEventKind::Connected(uuid),
            });
        }

        let mut devices_to_remove = Vec::new();

        for (id, device) in &mut self.devices {
            // TODO: Figure out how to merge events
            match device.device.fetch_events() {
                Ok(evs) => {
                    for ev in evs {
                        match ev.event_type() {
                            EventType::KEY => {
                                let code_raw = ev.code();
                                let code = KeyCode(code_raw);

                                let Some(scancode) = device.keycode_mapping.get(&code).cloned()
                                else {
                                    continue;
                                };

                                let pressed = ev.value() != 0;

                                if let Some(button) = mappings
                                    .get_button(device.device_id, scancode)
                                    .or_else(|| guess_button(code))
                                {
                                    events.push(GamepadEvent {
                                        id: GamepadId(*id),
                                        timestamp: ev.timestamp(),
                                        kind: GamepadEventKind::ButtonChanged(button, pressed),
                                    });
                                }
                            }
                            EventType::ABSOLUTE => {
                                let code_raw = ev.code();
                                let code = AbsoluteAxisCode(code_raw);

                                let mut was_hat = false;

                                let mut handle_hat =
                                    |x: AbsoluteAxisCode, y: AbsoluteAxisCode, i: HatIndex| {
                                        let value = ev.value();

                                        let hat_index = if code == x {
                                            &mut device.hatx
                                        } else if code == y {
                                            &mut device.haty
                                        } else {
                                            return;
                                        };

                                        was_hat = true;

                                        if let Some(button) = hat_index[i.to_index()] {
                                            events.push(GamepadEvent {
                                                id: GamepadId(*id),
                                                timestamp: ev.timestamp(),
                                                kind: GamepadEventKind::ButtonChanged(
                                                    button, false,
                                                ),
                                            });
                                        }

                                        let button = if code == y && value == -1 {
                                            HatButton::One
                                        } else if code == y && value == 1 {
                                            HatButton::Four
                                        } else if code == x && value == -1 {
                                            HatButton::Eight
                                        } else if code == x && value == 1 {
                                            HatButton::Two
                                        } else {
                                            hat_index[i.to_index()] = None;
                                            return;
                                        };

                                        let descriptor = HatDescriptor(i, button);

                                        let button = mappings
                                            .get_hat(device.device_id, descriptor)
                                            .unwrap_or_else(|| descriptor.guess_button());

                                        events.push(GamepadEvent {
                                            id: GamepadId(*id),
                                            timestamp: ev.timestamp(),
                                            kind: GamepadEventKind::ButtonChanged(button, true),
                                        });

                                        hat_index[i.to_index()] = Some(button);
                                    };

                                handle_hat(
                                    AbsoluteAxisCode::ABS_HAT0X,
                                    AbsoluteAxisCode::ABS_HAT0Y,
                                    HatIndex::Zero,
                                );
                                handle_hat(
                                    AbsoluteAxisCode::ABS_HAT1X,
                                    AbsoluteAxisCode::ABS_HAT1Y,
                                    HatIndex::One,
                                );
                                handle_hat(
                                    AbsoluteAxisCode::ABS_HAT2X,
                                    AbsoluteAxisCode::ABS_HAT2Y,
                                    HatIndex::Two,
                                );
                                handle_hat(
                                    AbsoluteAxisCode::ABS_HAT3X,
                                    AbsoluteAxisCode::ABS_HAT3Y,
                                    HatIndex::Three,
                                );

                                if was_hat {
                                    continue;
                                }

                                let Some((scancode, range)) =
                                    device.axis_mapping.get(&code).cloned()
                                else {
                                    continue;
                                };

                                let min = *range.start();
                                let max = *range.end();

                                let value = (ev.value() as f32 - min) / (max - min);

                                if let Some(axis) = mappings.get_axis(device.device_id, scancode) {
                                    events.push(GamepadEvent {
                                        id: GamepadId(*id),
                                        timestamp: ev.timestamp(),
                                        kind: GamepadEventKind::AxisMoved(
                                            axis.axis,
                                            axis.normalize(value),
                                        ),
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => {
                    events.push(GamepadEvent {
                        id: GamepadId(*id),
                        timestamp: SystemTime::now(),
                        kind: GamepadEventKind::Disconnected,
                    });
                    devices_to_remove.push(*id);
                }
            }
        }

        for id in devices_to_remove {
            if let Some(device) = self.devices.remove(&id) {
                self.device_paths.remove(&device.path);
            }
        }

        return Ok(());
    }
}
