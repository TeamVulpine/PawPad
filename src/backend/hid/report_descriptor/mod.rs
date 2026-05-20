use hidapi::{HidDevice, MAX_REPORT_DESCRIPTOR_SIZE};

use crate::backend::hid::{
    HidBackendError,
    report_descriptor::{
        global::ReportDescriptorCodeGlobal, local::ReportDescriptorCodeLocal,
        main::ReportDescriptorCodeMain,
    },
};

mod global;
mod local;
mod main;

pub struct HidReportDescriptor {}

struct HidReportDescriptorParser<'a> {
    data: &'a [u8],
    index: usize,
}

#[derive(Debug)]
enum ReportDescriptorCode {
    Main(ReportDescriptorCodeMain),
    Global(ReportDescriptorCodeGlobal),
    Local(ReportDescriptorCodeLocal),
    Long,
}

impl HidReportDescriptor {
    pub fn parse(device: &HidDevice) -> Result<Self, HidBackendError> {
        let mut buf = [0u8; MAX_REPORT_DESCRIPTOR_SIZE];
        let len = device.get_report_descriptor(&mut buf)?;

        let mut parser = HidReportDescriptorParser::new(&buf[..len]);

        while let Some(code) = ReportDescriptorCode::parse(&mut parser) {
            println!("{:?}", code);
        }

        return Ok(Self {});
    }
}

impl<'a> HidReportDescriptorParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        return Self { data, index: 0 };
    }

    fn next(&mut self) -> Option<u8> {
        let value = self.data.get(self.index).cloned()?;

        self.index += 1;

        return Some(value);
    }
}

impl ReportDescriptorCode {
    fn parse(reader: &mut HidReportDescriptorParser) -> Option<Self> {
        let first_byte = reader.next()?;

        if first_byte == 0xFE {
            let size = reader.next()?;
            let tag = reader.next()?;

            let mut data = Vec::<u8>::with_capacity(size as usize);

            for _ in 0..size {
                data.push(reader.next()?);
            }

            return Some(Self::Long {});
        }

        let size = first_byte & 0b11;
        let item_type = (first_byte >> 2) & 0b11;
        let tag = (first_byte >> 4) & 0b1111;

        let size = match size {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 4,
            _ => unreachable!(),
        };

        let mut bytes = [0u8; 4];

        for i in 0..size {
            bytes[i as usize] = reader.next()?;
        }

        let bytes = &bytes[..size];

        match item_type {
            0 => {
                let tag: ReportDescriptorCodeMain = ReportDescriptorCodeMain::from_tag(tag, bytes);

                return Some(Self::Main(tag));
            }
            1 => {
                let tag = ReportDescriptorCodeGlobal::from_tag(tag, bytes);

                return Some(Self::Global(tag));
            }
            2 => {
                let tag = ReportDescriptorCodeLocal::from_tag(tag, bytes);

                return Some(Self::Local(tag));
            }
            3 => {
                if tag != 0xF {
                    panic!("(HID ReportDescriptor) Unexpected reserved tag: {}", tag);
                }

                let size = bytes[0];
                let tag = bytes[1];

                let mut bytes = [0u8; 256];

                for i in 0..size {
                    bytes[i as usize] = reader.next()?;
                }

                let bytes = &bytes[..size as usize];

                return Some(Self::Long);
            }
            _ => unreachable!(),
        };
    }
}
