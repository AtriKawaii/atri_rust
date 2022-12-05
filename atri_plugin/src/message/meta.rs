use crate::message::MessageValue;

pub trait MetaMessage {
    fn metadata(&self) -> &MessageMetadata;
}

#[derive(Default, Clone)]
pub struct MessageMetadata {
    pub seqs: Vec<i32>,
    pub rands: Vec<i32>,
    pub time: i32,
    pub sender: i64,
    pub anonymous: Option<Anonymous>,
    pub reply: Option<Reply>,
}

impl MetaMessage for MessageMetadata {
    fn metadata(&self) -> &MessageMetadata {
        self
    }
}

#[derive(Default, Clone)]
pub struct Reply {
    pub reply_seq: i32,
    pub sender: i64,
    pub time: i32,
    pub elements: Vec<MessageValue>,
}

#[derive(Default, Debug, Clone)]
pub struct Anonymous {
    pub anon_id: Vec<u8>,
    pub nick: String,
    pub portrait_index: i32,
    pub bubble_index: i32,
    pub expire_time: i32,
    pub color: String,
}
