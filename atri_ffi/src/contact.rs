use crate::ManagedCloneable;

#[repr(C)]
pub struct FFIMember {
    pub is_named: bool,
    pub inner: ManagedCloneable,
}
