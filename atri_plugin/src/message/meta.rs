use crate::message::MessageElement;

#[derive(Debug, Clone, Default)]
pub struct MessageReceipt {
    pub seqs: Vec<i32>,
    pub rands: Vec<i32>,
    pub time: i64,
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

#[derive(Default, Clone)]
pub struct Reply {
    pub reply_seq: i32,
    pub sender: i64,
    pub time: i32,
    pub elements: Vec<MessageElement>,
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

mod ffi {
    use crate::message::meta::MessageReceipt;
    use atri_ffi::ffi::ForFFI;
    use atri_ffi::message::FFIMessageReceipt;

    impl ForFFI for MessageReceipt {
        type FFIValue = FFIMessageReceipt;

        fn into_ffi(self) -> Self::FFIValue {
            let MessageReceipt { seqs, rands, time } = self;

            FFIMessageReceipt {
                seqs: seqs.into(),
                rands: rands.into(),
                time,
            }
        }

        fn from_ffi(FFIMessageReceipt { seqs, rands, time }: Self::FFIValue) -> Self {
            Self {
                seqs: seqs.into_vec(),
                rands: rands.into_vec(),
                time,
            }
        }
    }
}
