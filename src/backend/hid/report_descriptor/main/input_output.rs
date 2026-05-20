#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataKind {
    Data,
    Constant,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayKind {
    Array,
    Variable,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionKind {
    Absolute,
    Relative,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapKind {
    NoWrap,
    Wrap,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinearKind {
    Linear,
    NonLinear,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferredKind {
    PreferredState,
    NoPreferred,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NullKind {
    NoNull,
    NullState,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolatileKind {
    NonVolatile,
    Volatile,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferKind {
    BitField,
    BufferedBytes,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct InputOutput {
    pub data_kind: DataKind,
    pub array_kind: ArrayKind,
    pub position_kind: PositionKind,
    pub wrap_kind: WrapKind,
    pub linear_kind: LinearKind,
    pub preferred_kind: PreferredKind,
    pub null_kind: NullKind,
    pub volatile_kind: VolatileKind,
    pub buffer_kind: BufferKind,
}

impl InputOutput {
    fn get_raw_flags(bytes: &[u8]) -> u16 {
        if bytes.len() == 1 {
            return bytes[0] as u16;
        }

        if bytes.len() == 0 {
            return 0;
        }

        return u16::from_be_bytes(*(bytes[..2]).as_array().unwrap());
    }

    fn from_u16(value: u16) -> Self {
        return Self {
            data_kind: if value & (1 << 0) == 0 {
                DataKind::Data
            } else {
                DataKind::Constant
            },

            array_kind: if value & (1 << 1) == 0 {
                ArrayKind::Array
            } else {
                ArrayKind::Variable
            },

            position_kind: if value & (1 << 2) == 0 {
                PositionKind::Absolute
            } else {
                PositionKind::Relative
            },

            wrap_kind: if value & (1 << 3) == 0 {
                WrapKind::NoWrap
            } else {
                WrapKind::Wrap
            },

            linear_kind: if value & (1 << 4) == 0 {
                LinearKind::Linear
            } else {
                LinearKind::NonLinear
            },

            preferred_kind: if value & (1 << 5) == 0 {
                PreferredKind::PreferredState
            } else {
                PreferredKind::NoPreferred
            },

            null_kind: if value & (1 << 6) == 0 {
                NullKind::NoNull
            } else {
                NullKind::NullState
            },

            volatile_kind: if value & (1 << 7) == 0 {
                VolatileKind::NonVolatile
            } else {
                VolatileKind::Volatile
            },

            buffer_kind: if value & (1 << 8) == 0 {
                BufferKind::BitField
            } else {
                BufferKind::BufferedBytes
            },
        };
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        return Self::from_u16(Self::get_raw_flags(bytes));
    }
}

impl std::fmt::Debug for InputOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;

        let mut first = true;

        let mut push = |f: &mut std::fmt::Formatter<'_>, v: &dyn std::fmt::Debug| {
            if !first {
                write!(f, " | ")?;
            }
            first = false;
            write!(f, "{:?}", v)
        };

        push(f, &self.data_kind)?;
        push(f, &self.array_kind)?;
        push(f, &self.position_kind)?;
        push(f, &self.wrap_kind)?;
        push(f, &self.linear_kind)?;
        push(f, &self.preferred_kind)?;
        push(f, &self.null_kind)?;
        push(f, &self.volatile_kind)?;
        push(f, &self.buffer_kind)?;

        return write!(f, ")");
    }
}
