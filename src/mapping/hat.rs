#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::gamepad::button::GamepadButton;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HatButton {
    One,
    Two,
    Four,
    Eight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HatIndex {
    Zero,
    One,
    Two,
    Three,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HatDescriptor(pub HatIndex, pub HatButton);

impl HatButton {
    pub fn to_char(&self) -> u8 {
        return match self {
            Self::One => b'1',
            Self::Two => b'2',
            Self::Four => b'4',
            Self::Eight => b'8',
        };
    }

    pub fn from_char(c: u8) -> Option<Self> {
        match c {
            b'1' => return Some(Self::One),
            b'2' => return Some(Self::Two),
            b'4' => return Some(Self::Four),
            b'8' => return Some(Self::Eight),
            _ => return None,
        }
    }
}

impl HatIndex {
    pub fn to_char(&self) -> u8 {
        return match self {
            Self::Zero => b'0',
            Self::One => b'1',
            Self::Two => b'2',
            Self::Three => b'3',
        };
    }

    pub fn from_char(c: u8) -> Option<Self> {
        match c {
            b'0' => return Some(Self::Zero),
            b'1' => return Some(Self::One),
            b'2' => return Some(Self::Two),
            b'3' => return Some(Self::Three),
            _ => return None,
        }
    }

    pub(crate) fn to_index(&self) -> usize {
        return match self {
            Self::Zero => 0,
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
        };
    }
}

impl HatDescriptor {
    pub(crate) fn guess_button(&self) -> GamepadButton {
        match self.1 {
            HatButton::One => GamepadButton::DPadUp,
            HatButton::Two => GamepadButton::DPadRight,
            HatButton::Four => GamepadButton::DPadDown,
            HatButton::Eight => GamepadButton::DPadLeft,
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for HatDescriptor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = [self.0.to_char(), b'.', self.1.to_char()];

        let string = std::str::from_utf8(&bytes).unwrap();

        return serializer.serialize_str(string);
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for HatDescriptor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        let bytes = s.as_bytes();

        if bytes.len() != 3 || bytes[1] != b'.' {
            return Err(serde::de::Error::custom(
                "expected format \"<index>.<button>\"",
            ));
        }

        let index = HatIndex::from_char(bytes[0])
            .ok_or_else(|| serde::de::Error::custom("invalid hat index"))?;

        let button = HatButton::from_char(bytes[2])
            .ok_or_else(|| serde::de::Error::custom("invalid hat button"))?;

        Ok(HatDescriptor(index, button))
    }
}
