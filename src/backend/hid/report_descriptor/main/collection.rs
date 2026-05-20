use crate::backend::hid::report_descriptor::parse_unsigned;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Collection {
    Physical,
    Application,
    Logical,
    Report,
    NamedArray,
    UsageSwitch,
    UsageModifier,

    Reserved(u8),
    VendorDefined(u8),
}

impl Collection {
    fn from_raw(value: u8) -> Self {
        match value {
            0x00 => Collection::Physical,
            0x01 => Collection::Application,
            0x02 => Collection::Logical,
            0x03 => Collection::Report,
            0x04 => Collection::NamedArray,
            0x05 => Collection::UsageSwitch,
            0x06 => Collection::UsageModifier,

            0x07..=0x7F => Collection::Reserved(value),
            0x80..=0xFF => Collection::VendorDefined(value),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        return Self::from_raw(parse_unsigned(bytes) as u8);
    }
}
