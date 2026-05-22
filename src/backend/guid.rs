/// Portions of this file are derived from SDL (Simple DirectMedia Layer).
///
/// SDL is licensed under the zlib license:
///
/// Copyright (C) 1997-2026 Sam Lantinga <slouken@libsdl.org>
///
/// This software is provided 'as-is', without any express or implied warranty.
/// In no event will the authors be held liable for any damages.
/// Permission is granted for any purpose, including commercial applications,
/// subject to the following restrictions:
///
/// 1. The origin of this software must not be misrepresented.
/// 2. Altered versions must be plainly marked as such.
/// 3. This notice must not be removed or altered from source distribution.
///
/// Full license text:
/// https://github.com/libsdl-org/SDL/blob/main/LICENSE.txt
use uuid::Uuid;

fn crc16(mut crc: u16, data: &[u8]) -> u16 {
    for &b in data {
        crc ^= b as u16;

        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    return crc;
}

/// Gets the GUID of a device using the same method as SDL.
/// This was chosen to allow the SDL controller mapping database to be easily ported to our needs.
pub fn get_guid(
    bus: u16,
    vendor: u16,
    product: u16,
    version: u16,
    vendor_name: Option<&str>,
    product_name: Option<&str>,
    driver_signature: u8,
    driver_data: u8,
) -> Uuid {
    let mut bytes = [0u8; 16];

    let mut crc = 0u16;

    match (vendor_name, product_name) {
        (Some(vn), Some(pn)) if !vn.is_empty() && !pn.is_empty() => {
            crc = crc16(crc, vn.as_bytes());
            crc = crc16(crc, b" ");
            crc = crc16(crc, pn.as_bytes());
        }
        (_, Some(pn)) => {
            crc = crc16(crc, pn.as_bytes());
        }
        _ => {}
    }

    bytes[0..2].copy_from_slice(&bus.to_le_bytes());
    bytes[2..4].copy_from_slice(&crc.to_le_bytes());

    if vendor != 0 {
        bytes[4..6].copy_from_slice(&vendor.to_le_bytes());

        bytes[8..10].copy_from_slice(&product.to_le_bytes());

        bytes[12..14].copy_from_slice(&version.to_le_bytes());

        bytes[14] = driver_signature;
        bytes[15] = driver_data;
    } else {
        let offset = 4;
        let mut available = 16 - offset;

        if driver_signature != 0 {
            available -= 2;
            bytes[14] = driver_signature;
            bytes[15] = driver_data;
        }

        if let Some(name) = product_name {
            let name_bytes = name.as_bytes();
            let len = name_bytes.len().min(available);
            bytes[offset..offset + len].copy_from_slice(&name_bytes[..len]);
        }
    }

    return Uuid::from_bytes(bytes);
}

pub fn alternative_guid(base: Uuid) -> Uuid {
    let mut bytes = base.into_bytes();

    bytes[13] = 1;

    bytes[2..4].fill(0);

    return Uuid::from_bytes(bytes);
}
