use crate::message::{MessageValue, PushMessage};

#[derive(Clone)]
pub struct At {
    pub target: i64,
    pub display: String,
}

impl PushMessage for At {
    fn push_to(self, v: &mut Vec<MessageValue>) {
        v.push(MessageValue::At(self));
    }
}
