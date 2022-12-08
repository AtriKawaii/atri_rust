pub mod at;
pub mod face;
mod ffi;
pub mod forward;
pub mod image;
pub mod macros;
pub mod meta;

use atri_ffi::{ManagedCloneable, RustStr};

use crate::error::{AtriError, AtriResult};
use crate::loader::get_plugin_manager_vtb;
use crate::message::at::At;
use crate::message::face::Face;
use crate::message::image::Image;
use crate::message::meta::{Anonymous, MessageMetadata, Reply};
use atri_ffi::ffi::ForFFI;
use std::fmt::Write;
use std::slice::Iter;
use std::{mem, vec};

#[derive(Default, Clone)]
pub struct MessageChain {
    meta: MessageMetadata,
    elements: Vec<MessageElement>,
}

impl MessageChain {
    pub fn builder() -> MessageChainBuilder {
        MessageChainBuilder::new()
    }

    pub fn iter(&self) -> Iter<MessageElement> {
        self.into_iter()
    }

    pub fn into_reply(self) -> Reply {
        Reply {
            reply_seq: self.meta.seqs[0],
            sender: self.meta.sender,
            time: self.meta.time,
            elements: self.elements,
        }
    }

    pub fn metadata(&self) -> &MessageMetadata {
        &self.meta
    }

    pub fn metadata_mut(&mut self) -> &mut MessageMetadata {
        &mut self.meta
    }

    pub fn with_reply(&mut self, reply: Reply) {
        self.metadata_mut().reply = Some(reply);
    }

    pub fn with_anonymous(&mut self, ano: Anonymous) {
        self.metadata_mut().anonymous = Some(ano);
    }

    pub fn to_json(&self) -> String {
        (get_plugin_manager_vtb().message_chain_to_json)(self.clone().into_ffi()).into()
    }

    pub fn from_json(json: &str) -> AtriResult<Self> {
        let rs = RustStr::from(json);
        let result = (get_plugin_manager_vtb().message_chain_from_json)(rs);
        Result::from(result)
            .map(MessageChain::from_ffi)
            .map_err(AtriError::SerializationError)
    }
}

impl IntoIterator for MessageChain {
    type Item = MessageElement;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'a> IntoIterator for &'a MessageChain {
    type Item = &'a MessageElement;
    type IntoIter = Iter<'a, MessageElement>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}

impl ToString for MessageChain {
    fn to_string(&self) -> String {
        let mut s = String::new();
        for value in self {
            value.push_to_string(&mut s);
        }
        s
    }
}

#[derive(Clone)]
pub enum MessageElement {
    Text(String),
    Image(Image),
    At(At),
    AtAll,
    Face(Face),
    Unknown(ManagedCloneable),
}

impl MessageElement {
    fn push_to_string(&self, str: &mut String) {
        match self {
            Self::Text(text) => str.push_str(text),
            Self::Image(img) => {
                let _ = write!(str, "$[Image:{}]", img.url());
            }
            Self::At(At { target, display }) => {
                let _ = write!(str, "$[At:{}({})]", display, target);
            }
            Self::AtAll => str.push_str("$[AtAll]"),
            Self::Face(face) => {
                let _ = write!(str, "$[Face:{}]", face.name);
            }
            Self::Unknown(_) => {}
        }
    }
}

impl ToString for MessageElement {
    fn to_string(&self) -> String {
        let mut s = String::new();
        self.push_to_string(&mut s);
        s
    }
}

#[derive(Default)]
pub struct MessageChainBuilder {
    anonymous: Option<Anonymous>,
    reply: Option<Reply>,
    value: Vec<MessageElement>,
    buf: String,
}

impl MessageChainBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push<E: PushMessage>(&mut self, elem: E) -> &mut Self {
        self.flush();
        elem.push_to(&mut self.value);
        self
    }

    pub fn push_str(&mut self, str: &str) -> &mut Self {
        self.buf.push_str(str);
        self
    }

    pub fn build(mut self) -> MessageChain {
        self.flush();
        MessageChain {
            elements: self.value,
            meta: MessageMetadata {
                seqs: vec![],
                rands: vec![],
                time: 0,
                sender: 0,
                anonymous: self.anonymous,
                reply: self.reply,
            },
        }
    }

    pub fn with_reply(&mut self, reply: Reply) {
        self.reply = Some(reply);
    }

    pub fn with_anonymous(&mut self, ano: Anonymous) {
        self.anonymous = Some(ano);
    }

    fn flush(&mut self) {
        let buf = mem::take(&mut self.buf);
        let text = MessageElement::Text(buf);
        self.value.push(text);
    }
}

pub trait PushMessage {
    fn push_to(self, v: &mut Vec<MessageElement>);
}

impl<M: PushMessage> From<M> for MessageChain {
    fn from(push: M) -> Self {
        let mut chain = MessageChain::default();
        push.push_to(&mut chain.elements);
        chain
    }
}

impl PushMessage for MessageElement {
    fn push_to(self, v: &mut Vec<MessageElement>) {
        v.push(self);
    }
}

impl PushMessage for String {
    fn push_to(self, v: &mut Vec<MessageElement>) {
        v.push(MessageElement::Text(self));
    }
}

impl PushMessage for &str {
    fn push_to(self, v: &mut Vec<MessageElement>) {
        String::from(self).push_to(v);
    }
}

impl PushMessage for MessageChainBuilder {
    fn push_to(self, v: &mut Vec<MessageElement>) {
        let chain = self.build();
        v.extend(chain.elements);
    }
}
