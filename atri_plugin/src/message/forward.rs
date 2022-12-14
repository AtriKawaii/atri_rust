use crate::message::MessageChain;

#[derive(Default, Clone)]
pub struct ForwardMessage(Vec<ForwardNode>);

impl ForwardMessage {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<ForwardNode> {
        self.0.iter()
    }

    pub fn push(&mut self, node: ForwardNode) {
        self.0.push(node);
    }

    pub fn push_node<I: IntoForwardNode>(&mut self, info: ForwardNodeInfo, node: I) {
        self.push(node.into_node(info))
    }

    pub fn push_message<M: Into<MessageChain>>(&mut self, info: ForwardNodeInfo, msg: M) {
        self.push(ForwardNode::NormalMessage {
            info,
            chain: msg.into(),
        })
    }

    pub fn push_forward<M: Into<ForwardMessage>>(&mut self, info: ForwardNodeInfo, msg: M) {
        self.push(ForwardNode::ForwardMessage {
            info,
            forward: msg.into(),
        })
    }
}

pub trait IntoForwardNode {
    fn into_node(self, info: ForwardNodeInfo) -> ForwardNode;
}

impl IntoForwardNode for MessageChain {
    fn into_node(self, info: ForwardNodeInfo) -> ForwardNode {
        ForwardNode::NormalMessage { info, chain: self }
    }
}

impl IntoForwardNode for ForwardMessage {
    fn into_node(self, info: ForwardNodeInfo) -> ForwardNode {
        ForwardNode::ForwardMessage {
            info,
            forward: self,
        }
    }
}

impl IntoForwardNode for ForwardNode {
    fn into_node(self, _: ForwardNodeInfo) -> ForwardNode {
        self
    }
}

impl IntoIterator for ForwardMessage {
    type Item = ForwardNode;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Clone)]
pub enum ForwardNode {
    NormalMessage {
        info: ForwardNodeInfo,
        chain: MessageChain,
    },
    ForwardMessage {
        info: ForwardNodeInfo,
        forward: ForwardMessage,
    },
}

#[derive(Default, Debug, Clone)]
pub struct ForwardNodeInfo {
    pub sender_id: i64,
    pub sender_name: String,
    pub time: i32,
}

mod ffi {
    use super::{ForwardMessage, ForwardNode, ForwardNodeInfo};
    use crate::message::MessageChain;
    use atri_ffi::ffi::ForFFI;
    use atri_ffi::message::forward::{FFIForwardNode, FFIForwardNodeInfo, ForwardNodeUnion};
    use atri_ffi::RustVec;
    use std::mem::ManuallyDrop;

    impl ForFFI for ForwardNode {
        type FFIValue = FFIForwardNode;

        fn into_ffi(self) -> Self::FFIValue {
            match self {
                Self::NormalMessage { info, chain } => FFIForwardNode {
                    is_normal: true,
                    info: info.into_ffi(),
                    inner: ForwardNodeUnion {
                        normal: ManuallyDrop::new(chain.into_ffi()),
                    },
                },
                Self::ForwardMessage { info, forward: msg } => FFIForwardNode {
                    is_normal: false,
                    info: info.into_ffi(),
                    inner: ForwardNodeUnion {
                        forward: ManuallyDrop::new(msg.into_ffi()),
                    },
                },
            }
        }

        fn from_ffi(
            FFIForwardNode {
                is_normal,
                info,
                inner,
            }: Self::FFIValue,
        ) -> Self {
            unsafe {
                if is_normal {
                    Self::NormalMessage {
                        info: ForwardNodeInfo::from_ffi(info),
                        chain: MessageChain::from_ffi(ManuallyDrop::into_inner(inner.normal)),
                    }
                } else {
                    Self::ForwardMessage {
                        info: ForwardNodeInfo::from_ffi(info),
                        forward: ForwardMessage::from_ffi(ManuallyDrop::into_inner(inner.forward)),
                    }
                }
            }
        }
    }

    impl ForFFI for ForwardNodeInfo {
        type FFIValue = FFIForwardNodeInfo;

        fn into_ffi(self) -> Self::FFIValue {
            let ForwardNodeInfo {
                sender_id,
                sender_name,
                time,
            } = self;

            FFIForwardNodeInfo {
                sender_id,
                sender_name: sender_name.into(),
                time,
            }
        }

        fn from_ffi(
            FFIForwardNodeInfo {
                sender_id,
                sender_name,
                time,
            }: Self::FFIValue,
        ) -> Self {
            Self {
                sender_id,
                sender_name: sender_name.into(),
                time,
            }
        }
    }

    impl ForFFI for ForwardMessage {
        type FFIValue = RustVec<FFIForwardNode>;

        fn into_ffi(self) -> Self::FFIValue {
            self.0
                .into_iter()
                .map(ForwardNode::into_ffi)
                .collect::<Vec<FFIForwardNode>>()
                .into()
        }

        fn from_ffi(rs: Self::FFIValue) -> Self {
            Self(
                rs.into_vec()
                    .into_iter()
                    .map(ForwardNode::from_ffi)
                    .collect(),
            )
        }
    }
}
