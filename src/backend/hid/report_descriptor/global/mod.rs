use crate::backend::hid::report_descriptor::{global::unit::Unit, parse_unsigned};

pub mod unit;

#[derive(Debug)]
pub enum ReportDescriptorCodeGlobal {
    UsagePage(u16),
    LogicalMinimum(u8),
    LogicalMaximum(u8),
    PhysicalMinimum(u8),
    PhysicalMaximum(u8),
    UnitExponent(u32),
    Unit(Option<Unit>),
    ReportSize(u32),
    ReportId(u32),
    ReportCount(u32),
    Push,
    Pop,
    Reserved(u8),
}

impl ReportDescriptorCodeGlobal {
    pub fn from_tag(tag: u8, bytes: &[u8]) -> Self {
        match tag {
            0 => Self::UsagePage(parse_unsigned(bytes) as u16),
            1 => Self::LogicalMinimum(parse_unsigned(bytes) as u8),
            2 => Self::LogicalMaximum(parse_unsigned(bytes) as u8),
            3 => Self::PhysicalMinimum(parse_unsigned(bytes) as u8),
            4 => Self::PhysicalMaximum(parse_unsigned(bytes) as u8),
            5 => Self::UnitExponent(parse_unsigned(bytes)),
            6 => Self::Unit(Unit::from_bytes(bytes)),
            7 => Self::ReportSize(parse_unsigned(bytes)),
            8 => Self::ReportId(parse_unsigned(bytes)),
            9 => Self::ReportCount(parse_unsigned(bytes)),
            10 => Self::Push,
            11 => Self::Pop,
            _ => Self::Reserved(tag),
        }
    }
}
