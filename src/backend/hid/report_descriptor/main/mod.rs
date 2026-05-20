use crate::backend::hid::report_descriptor::main::{
    collection::Collection, input_output::InputOutput,
};

pub mod collection;
pub mod input_output;

#[derive(Debug)]
pub enum ReportDescriptorCodeMain {
    Input(InputOutput),
    Output(InputOutput),
    Feature(InputOutput),
    Collection(Collection),
    EndCollection,
    Reserved(u8),
}

impl ReportDescriptorCodeMain {
    pub fn from_tag(tag: u8, bytes: &[u8]) -> Self {
        match tag {
            8 => Self::Input(InputOutput::from_bytes(bytes)),
            9 => Self::Output(InputOutput::from_bytes(bytes)),
            10 => Self::Feature(InputOutput::from_bytes(bytes)),
            11 => Self::Collection(Collection::from_bytes(bytes)),
            12 => Self::EndCollection,
            _ => Self::Reserved(tag),
        }
    }
}
