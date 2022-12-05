pub mod at;
mod ffi;
pub mod image;
pub mod meta;

use atri_ffi::Managed;

use crate::message::at::At;
use crate::message::image::Image;
use crate::message::meta::{Anonymous, MessageMetadata, MetaMessage, Reply};
use std::slice::Iter;
use std::{mem, vec};

#[derive(Default)]
pub struct MessageChain {
    meta: MessageMetadata,
    elements: Vec<MessageValue>,
}

impl MessageChain {
    pub fn builder() -> MessageChainBuilder {
        MessageChainBuilder::new()
    }

    pub fn iter(&self) -> Iter<MessageValue> {
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
}

impl MetaMessage for MessageChain {
    fn metadata(&self) -> &MessageMetadata {
        &self.meta
    }
}

impl IntoIterator for MessageChain {
    type Item = MessageValue;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'a> IntoIterator for &'a MessageChain {
    type Item = &'a MessageValue;
    type IntoIter = Iter<'a, MessageValue>;

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

pub enum MessageValue {
    Text(String),
    Image(Image),
    At(At),
    AtAll,
    Unknown(Managed),
}

impl MessageValue {
    fn push_to_string(&self, str: &mut String) {
        match self {
            Self::Text(text) => str.push_str(text),
            Self::Image(img) => str.push_str(&format!("[Image:{}]", img.url())),
            Self::At(At { target, display }) => {
                str.push_str(&format!("[At:{}({})]", target, display))
            }
            Self::AtAll => str.push_str("[AtAll]"),
            Self::Unknown(_) => {}
        }
    }
}

impl ToString for MessageValue {
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
    value: Vec<MessageValue>,
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
        let text = MessageValue::Text(buf);
        self.value.push(text);
    }
}

pub struct MessageReceipt(pub(crate) Managed);

pub trait PushMessage {
    fn push_to(self, v: &mut Vec<MessageValue>);
}

impl<M: PushMessage> From<M> for MessageChain {
    fn from(push: M) -> Self {
        let mut chain = MessageChain::default();
        push.push_to(&mut chain.elements);
        chain
    }
}

impl PushMessage for MessageValue {
    fn push_to(self, v: &mut Vec<MessageValue>) {
        v.push(self);
    }
}

impl PushMessage for String {
    fn push_to(self, v: &mut Vec<MessageValue>) {
        v.push(MessageValue::Text(self));
    }
}

impl PushMessage for &str {
    fn push_to(self, v: &mut Vec<MessageValue>) {
        String::from(self).push_to(v);
    }
}

impl PushMessage for MessageChainBuilder {
    fn push_to(self, v: &mut Vec<MessageValue>) {
        let chain = self.build();
        v.extend(chain.elements);
    }
}
