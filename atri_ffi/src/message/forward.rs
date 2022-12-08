use crate::message::FFIMessageChain;
use crate::{RustString, RustVec};
use std::mem::ManuallyDrop;

#[repr(C)]
pub struct FFIForwardNodeInfo {
    pub sender_id: i64,
    pub sender_name: RustString,
    pub time: i32,
}

pub struct FFIForwardNode {
    pub is_normal: bool,
    pub info: FFIForwardNodeInfo,
    pub inner: ForwardNodeUnion,
}

#[repr(C)]
pub union ForwardNodeUnion {
    pub normal: ManuallyDrop<FFIMessageChain>,
    pub forward: ManuallyDrop<RustVec<FFIForwardNode>>,
}
