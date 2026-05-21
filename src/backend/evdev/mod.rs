use std::{
    collections::{HashMap, HashSet},
    fs::{self},
    io,
    ops::RangeInclusive,
    path::PathBuf,
    time::SystemTime,
};

use evdev::{AbsoluteAxisCode, BusType, Device, EventType, KeyCode};
use pawkit_crockford::Ulid;
use thiserror::Error;

use crate::{
    gamepad::{GamepadEvent, GamepadEventKind, GamepadId, button::GamepadButton},
    mapping::{BakedGamepadMappings, Transport, device_id::DeviceId},
};

struct MappedDevice {
    device: Device,
    transport: Transport,
    device_id: DeviceId,
    keycode_mapping: HashMap<KeyCode, u16>,
    axis_mapping: HashMap<AbsoluteAxisCode, (u16, RangeInclusive<f32>)>,
    hatx: Option<GamepadButton>,
    haty: Option<GamepadButton>,
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

            // Commented out for testing purposes
            // This is the code that checks for hidraw devices
            // let path = Path::new("/sys/class/input")
            //     .join(file.file_name())
            //     .join("device/device/hidraw");

            // if path.exists() {
            //     let Ok(read_dir) = fs::read_dir(path) else {
            //         continue;
            //     };

            //     let mut skip_gamepad = false;

            //     for file in read_dir {
            //         let Ok(file) = file else {
            //             continue;
            //         };

            //         let path = Path::new("/dev").join(file.file_name());

            //         match File::open(path) {
            //             Ok(_) => {
            //                 skip_gamepad = true;
            //             }
            //             _ => {}
            //         }
            //     }

            //     if skip_gamepad {
            //         continue;
            //     }
            // }

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

            let id = device.input_id();

            let device_id = DeviceId {
                vendor: id.vendor(),
                product: id.product(),
                version: id.version(),
            };

            let transport = match id.bus_type() {
                BusType::BUS_USB => Transport::Usb,
                BusType::BUS_BLUETOOTH => Transport::Bluetooth,
                it => panic!(
                    "Unsupported transport {:?}. Open an issue at https://github.com/TeamVulpine/PawPad/",
                    it
                ),
            };

            let id = Ulid::new();

            let mapped = MappedDevice {
                keycode_mapping: build_keycode_mapping(&device),
                axis_mapping: build_axis_mapping(&device)?,
                device,
                device_id,
                transport,
                hatx: None,
                haty: None,
            };

            self.devices.insert(id, mapped);
            self.device_paths.insert(file.path());

            events.push(GamepadEvent {
                id: GamepadId(id),
                timestamp: SystemTime::now(),
                kind: GamepadEventKind::Connected(device_id, transport),
            });
        }

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

                                if let Some(button) = mappings.get_button(
                                    device.transport,
                                    device.device_id,
                                    scancode,
                                ) {
                                    events.push(GamepadEvent {
                                        id: GamepadId(*id),
                                        timestamp: ev.timestamp(),
                                        kind: GamepadEventKind::ButtonChanged(button, pressed),
                                    });
                                } else {
                                    events.push(GamepadEvent {
                                        id: GamepadId(*id),
                                        timestamp: ev.timestamp(),
                                        kind: GamepadEventKind::UnknwonButtonChanged(scancode, pressed),
                                    });
                                }
                            }
                            EventType::ABSOLUTE => {
                                let code_raw = ev.code();
                                let code = AbsoluteAxisCode(code_raw);

                                let mut was_hat = false;

                                if let Some(hat) = mappings.get_hat(device.transport, device.device_id) {
                                    let hatx = AbsoluteAxisCode(AbsoluteAxisCode::ABS_HAT0X.0 + hat as u16 * 2);
                                    let haty = AbsoluteAxisCode(hatx.0 + 1);

                                    if code == hatx {
                                        let value = ev.value();

                                        if let Some(hat) = device.hatx {
                                            events.push(GamepadEvent {
                                                id: GamepadId(*id),
                                                timestamp: ev.timestamp(),
                                                kind: GamepadEventKind::ButtonChanged(hat, false),
                                            });
                                        }

                                        let value = if value == -1 {
                                            Some(GamepadButton::DPadLeft)
                                        } else if value == 1 {
                                            Some(GamepadButton::DPadRight)
                                        } else {
                                            None
                                        };
                                        
                                        if let Some(hat) = value {
                                            events.push(GamepadEvent {
                                                id: GamepadId(*id),
                                                timestamp: ev.timestamp(),
                                                kind: GamepadEventKind::ButtonChanged(hat, true),
                                            });
                                        }

                                        device.hatx = value;

                                        was_hat = true;
                                    }

                                    if code == haty {
                                        let value = ev.value();

                                        if let Some(hat) = device.haty {
                                            events.push(GamepadEvent {
                                                id: GamepadId(*id),
                                                timestamp: ev.timestamp(),
                                                kind: GamepadEventKind::ButtonChanged(hat, false),
                                            });
                                        }

                                        let value = if value == -1 {
                                            Some(GamepadButton::DPadUp)
                                        } else if value == 1 {
                                            Some(GamepadButton::DPadDown)
                                        } else {
                                            None
                                        };
                                        
                                        if let Some(hat) = value {
                                            events.push(GamepadEvent {
                                                id: GamepadId(*id),
                                                timestamp: ev.timestamp(),
                                                kind: GamepadEventKind::ButtonChanged(hat, true),
                                            });
                                        }

                                        device.haty = value;

                                        was_hat = true;
                                    }
                                }

                                let Some((scancode, range)) =
                                    device.axis_mapping.get(&code).cloned()
                                else {
                                    continue;
                                };

                                let min = *range.start();
                                let max = *range.end();

                                let value = (ev.value() as f32 - min) / (max - min);

                                if let Some(axis) =
                                    mappings.get_axis(device.transport, device.device_id, scancode)
                                {
                                    events.push(GamepadEvent {
                                        id: GamepadId(*id),
                                        timestamp: ev.timestamp(),
                                        kind: GamepadEventKind::AxisMoved(
                                            axis,
                                            axis.normalize(value),
                                        ),
                                    });
                                } else if !was_hat {
                                    events.push(GamepadEvent {
                                        id: GamepadId(*id),
                                        timestamp: ev.timestamp(),
                                        kind: GamepadEventKind::UnknownAxisMoved(
                                            scancode,
                                            value,
                                        ),
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    panic!("{e}");
                }
            }
        }

        return Ok(());
    }
}
