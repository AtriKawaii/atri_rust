use crate::message::meta::FFIMessageMetadata;
use crate::{Managed, ManagedCloneable, RustString, RustVec};
use std::mem::ManuallyDrop;

pub mod meta;

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum MessageElementFlag {
    Text = 0,
    Image = 1,
    At = 2,
    AtAll = 3,
    Unknown = 255,
}

impl MessageElementFlag {
    #[inline]
    pub fn value(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for MessageElementFlag {
    type Error = u8;

    fn try_from(flag: u8) -> Result<Self, Self::Error> {
        Ok(match flag {
            0 => Self::Text,
            1 => Self::Image,
            2 => Self::At,
            3 => Self::AtAll,
            255 => Self::Unknown,
            _ => return Err(flag),
        })
    }
}

#[repr(C)]
pub struct FFIMessageChain {
    pub meta: FFIMessageMetadata,
    pub inner: RustVec<FFIMessageValue>,
}

#[repr(C)]
pub struct FFIMessageValue {
    pub t: u8,
    pub union: MessageElementUnion,
}

#[repr(C)]
pub union MessageElementUnion {
    pub text: ManuallyDrop<RustString>,
    pub image: ManuallyDrop<ManagedCloneable>,
    pub at: ManuallyDrop<FFIAt>,
    pub at_all: (),
    pub unknown: ManuallyDrop<ManagedCloneable>,
}

#[repr(C)]
pub struct FFIImage {
    pub t: u8,
    pub union: ImageUnion,
}

#[repr(C)]
pub union ImageUnion {
    pub group: ManuallyDrop<Managed>,
    pub friend: ManuallyDrop<Managed>,
}

#[repr(C)]
pub struct FFIAt {
    pub target: i64,
    pub display: RustString,
}
