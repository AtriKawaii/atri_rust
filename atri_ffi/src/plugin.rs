use crate::{Managed, RustStr};

#[repr(C)]
pub struct PluginInstance {
    pub instance: Managed,
    pub should_drop: bool,
    pub vtb: PluginVTable,
    pub abi_ver: u8,
    pub name: RustStr,
}

#[repr(C)]
pub struct PluginVTable {
    pub new: extern "C" fn() -> *mut (),
    pub enable: extern "C" fn(*mut ()),
    pub disable: extern "C" fn(*mut ()),
}

pub const fn abi_version() -> u8 {
    include!("../abi-version")
}
