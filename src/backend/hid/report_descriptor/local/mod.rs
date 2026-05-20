use crate::backend::hid::report_descriptor::parse_unsigned;

#[derive(Debug)]
pub enum ReportDescriptorCodeLocal {
    Usage(u32),
    UsageMinimum,
    UsageMaximum,
    DesignatorIndex,
    DesignatorMinimum,
    DesignatorMaximum,
    StringIndex,
    StringMinimum,
    StringMaximum,
    Delimiter,
    Reserved(u8),
}

impl ReportDescriptorCodeLocal {
    pub fn from_tag(tag: u8, bytes: &[u8]) -> Self {
        match tag {
            0 => Self::Usage(parse_unsigned(bytes)),
            1 => Self::UsageMinimum,
            2 => Self::UsageMaximum,
            3 => Self::DesignatorIndex,
            4 => Self::DesignatorMinimum,
            5 => Self::DesignatorMaximum,
            6 => Self::StringIndex,
            7 => Self::StringMinimum,
            8 => Self::StringMaximum,
            9 => Self::Delimiter,
            _ => Self::Reserved(tag),
        }
    }
}
