#[derive(Debug)]
pub enum ReportDescriptorCodeGlobal {
    UsagePage,
    LogicalMinimum,
    LogicalMaximum,
    PhysicalMinimum,
    PhysicalMaximum,
    UnitExponent,
    Unit,
    ReportSize,
    ReportId,
    ReportCount,
    Push,
    Pop,
    Reserved(u8),
}

impl ReportDescriptorCodeGlobal {
    pub fn from_tag(tag: u8, bytes: &[u8]) -> Self {
        match tag {
            0 => Self::UsagePage,
            1 => Self::LogicalMinimum,
            2 => Self::LogicalMaximum,
            3 => Self::PhysicalMinimum,
            4 => Self::PhysicalMaximum,
            5 => Self::UnitExponent,
            6 => Self::Unit,
            7 => Self::ReportSize,
            8 => Self::ReportId,
            9 => Self::ReportCount,
            10 => Self::Push,
            11 => Self::Pop,
            _ => Self::Reserved(tag),
        }
    }
}
