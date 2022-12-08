use crate::message::at::At;
use crate::message::face::Face;
use crate::message::image::Image;
use crate::message::meta::{Anonymous, MessageMetadata, Reply};
use crate::message::{MessageChain, MessageElement};
use atri_ffi::ffi::ForFFI;
use atri_ffi::message::meta::{
    FFIAnonymous, FFIMessageMetadata, FFIReply, ANONYMOUS_FLAG, NONE_META, REPLY_FLAG,
};
use atri_ffi::message::{
    FFIAt, FFIFace, FFIMessageChain, FFIMessageElement, MessageElementFlag, MessageElementUnion,
};
use atri_ffi::{RustString, RustVec};
use std::mem::{ManuallyDrop, MaybeUninit};

impl ForFFI for MessageChain {
    type FFIValue = FFIMessageChain;

    fn into_ffi(self) -> Self::FFIValue {
        let v: Vec<FFIMessageElement> = self
            .elements
            .into_iter()
            .map(MessageElement::into_ffi)
            .collect();

        let raw = RustVec::from(v);
        FFIMessageChain {
            meta: self.meta.into_ffi(),
            inner: raw,
        }
    }

    fn from_ffi(ffi: Self::FFIValue) -> Self {
        let v = ffi.inner.into_vec();
        let value = v.into_iter().map(MessageElement::from_ffi).collect();
        Self {
            meta: MessageMetadata::from_ffi(ffi.meta),
            elements: value,
        }
    }
}

impl ForFFI for MessageElement {
    type FFIValue = FFIMessageElement;

    fn into_ffi(self) -> Self::FFIValue {
        match self {
            MessageElement::Text(s) => FFIMessageElement {
                t: MessageElementFlag::Text.value(),
                union: MessageElementUnion {
                    text: ManuallyDrop::new(RustString::from(s)),
                },
            },
            MessageElement::Image(img) => FFIMessageElement {
                t: MessageElementFlag::Image.value(),
                union: MessageElementUnion {
                    image: ManuallyDrop::new(img.0),
                },
            },
            MessageElement::At(At { target, display }) => FFIMessageElement {
                t: MessageElementFlag::At.value(),
                union: MessageElementUnion {
                    at: ManuallyDrop::new({
                        let display = RustString::from(display);
                        FFIAt { target, display }
                    }),
                },
            },
            MessageElement::AtAll => FFIMessageElement {
                t: MessageElementFlag::AtAll.value(),
                union: MessageElementUnion { at_all: () },
            },
            MessageElement::Face(Face { index, name }) => FFIMessageElement {
                t: MessageElementFlag::Face.value(),
                union: MessageElementUnion {
                    face: ManuallyDrop::new({
                        let name = RustString::from(name);
                        FFIFace { index, name }
                    }),
                },
            },
            MessageElement::Unknown(ma) => FFIMessageElement {
                t: 255,
                union: MessageElementUnion {
                    unknown: ManuallyDrop::new(ma),
                },
            },
        }
    }

    fn from_ffi(value: Self::FFIValue) -> Self {
        unsafe {
            match MessageElementFlag::try_from(value.t).unwrap_or_else(|t| {
                panic!(
                    "Unknown message flag: {}, please update your atri_plugin crate",
                    t
                )
            }) {
                MessageElementFlag::Text => {
                    Self::Text(ManuallyDrop::into_inner(value.union.text).into())
                }
                MessageElementFlag::Image => {
                    Self::Image(Image(ManuallyDrop::into_inner(value.union.image)))
                }
                MessageElementFlag::At => {
                    let FFIAt { target, display } = ManuallyDrop::into_inner(value.union.at);
                    let display = String::from(display);

                    Self::At(At { target, display })
                }
                MessageElementFlag::AtAll => Self::AtAll,
                MessageElementFlag::Face => {
                    let FFIFace { index, name } = ManuallyDrop::into_inner(value.union.face);
                    let name = String::from(name);

                    Self::Face(Face { index, name })
                }
                MessageElementFlag::Unknown => {
                    Self::Unknown(ManuallyDrop::into_inner(value.union.unknown))
                }
            }
        }
    }
}

impl ForFFI for At {
    type FFIValue = FFIAt;

    fn into_ffi(self) -> Self::FFIValue {
        let At { target, display } = self;

        FFIAt {
            target,
            display: RustString::from(display),
        }
    }

    fn from_ffi(value: Self::FFIValue) -> Self {
        let FFIAt { target, display } = value;

        Self {
            target,
            display: String::from(display),
        }
    }
}

impl ForFFI for MessageMetadata {
    type FFIValue = FFIMessageMetadata;

    fn into_ffi(self) -> Self::FFIValue {
        let Self {
            seqs,
            rands,
            time,
            sender,
            anonymous,
            reply,
        } = self;

        let mut flags = NONE_META;

        let mut ffi_anonymous = MaybeUninit::uninit();
        if let Some(ano) = anonymous {
            flags |= ANONYMOUS_FLAG;
            ffi_anonymous.write(ano.into_ffi());
        }

        let mut ffi_reply = MaybeUninit::uninit();
        if let Some(reply) = reply {
            flags |= REPLY_FLAG;
            ffi_reply.write(reply.into_ffi());
        }

        FFIMessageMetadata {
            seqs: seqs.into(),
            rands: rands.into(),
            time,
            sender,
            flags,
            anonymous: ffi_anonymous,
            reply: ffi_reply,
        }
    }

    fn from_ffi(ffi: Self::FFIValue) -> Self {
        let FFIMessageMetadata {
            seqs,
            rands,
            time,
            sender,
            flags,
            reply,
            anonymous,
        } = ffi;

        unsafe {
            Self {
                seqs: seqs.into_vec(),
                rands: rands.into_vec(),
                time,
                sender,
                anonymous: if flags & ANONYMOUS_FLAG != 0 {
                    Some(Anonymous::from_ffi(anonymous.assume_init()))
                } else {
                    None
                },
                reply: if flags & REPLY_FLAG != 0 {
                    Some(Reply::from_ffi(reply.assume_init()))
                } else {
                    None
                },
            }
        }
    }
}

impl ForFFI for Reply {
    type FFIValue = FFIReply;

    fn into_ffi(self) -> Self::FFIValue {
        let Self {
            reply_seq,
            sender,
            time,
            elements,
        } = self;

        let ffi_value: Vec<FFIMessageElement> =
            elements.into_iter().map(|value| value.into_ffi()).collect();
        let raw = RustVec::from(ffi_value);

        FFIReply {
            reply_seq,
            sender,
            time,
            elements: raw,
        }
    }

    fn from_ffi(ffi: Self::FFIValue) -> Self {
        let FFIReply {
            reply_seq,
            sender,
            time,
            elements,
        } = ffi;

        let v = elements.into_vec();
        let value: Vec<MessageElement> = v.into_iter().map(MessageElement::from_ffi).collect();

        Self {
            reply_seq,
            sender,
            time,
            elements: value,
        }
    }
}

impl ForFFI for Anonymous {
    type FFIValue = FFIAnonymous;

    fn into_ffi(self) -> Self::FFIValue {
        let Self {
            anon_id,
            nick,
            portrait_index,
            bubble_index,
            expire_time,
            color,
        } = self;

        let anon_id = RustVec::from(anon_id);
        let nick = RustString::from(nick);
        let color = RustString::from(color);

        FFIAnonymous {
            anon_id,
            nick,
            portrait_index,
            bubble_index,
            expire_time,
            color,
        }
    }

    fn from_ffi(ffi: Self::FFIValue) -> Self {
        let FFIAnonymous {
            anon_id,
            nick,
            portrait_index,
            bubble_index,
            expire_time,
            color,
        } = ffi;

        let anon_id = anon_id.into_vec();
        let nick = String::from(nick);
        let color = String::from(color);

        Self {
            anon_id,
            nick,
            portrait_index,
            bubble_index,
            expire_time,
            color,
        }
    }
}
