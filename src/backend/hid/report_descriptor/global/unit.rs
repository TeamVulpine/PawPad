use crate::backend::hid::report_descriptor::parse_unsigned;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Centimeter,
    Radian,
    Inch,
    Degree,
    Gram,
    Slug,
    Second,
    Kelvin,
    Fahrenheit,
    Ampere,
    Candela,
}

impl Unit {
    fn from_raw(value: u8) -> Option<Self> {
        let system_raw = value & 0x0F;
        let kind_raw = (value >> 4) & 0x0F;

        let unit = match (kind_raw, system_raw) {
            (1, 0x1) => Self::Centimeter,
            (1, 0x2) => Self::Radian,
            (1, 0x3) => Self::Inch,
            (1, 0x4) => Self::Degree,

            (2, 0x1) | (2, 0x2) => Self::Gram,
            (2, 0x3) | (2, 0x4) => Self::Slug,

            (3, 0x1) | (3, 0x2) | (3, 0x3) | (3, 0x4) => Self::Second,

            (4, 0x1) | (4, 0x2) => Self::Kelvin,
            (4, 0x3) | (4, 0x4) => Self::Fahrenheit,

            (5, 0x1) | (5, 0x2) | (5, 0x3) | (5, 0x4) => Self::Ampere,

            (6, 0x1) | (6, 0x2) | (6, 0x3) | (6, 0x4) => Self::Candela,

            _ => return None,
        };

        return Some(unit);
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        return Self::from_raw(parse_unsigned(bytes) as u8);
    }
}
