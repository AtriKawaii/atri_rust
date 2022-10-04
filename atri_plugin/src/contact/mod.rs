use crate::contact::friend::Friend;
use crate::contact::group::Group;
use crate::contact::member::Member;
use crate::error::AtriError;
use crate::message::image::Image;
use crate::message::{MessageChain, MessageReceipt};

pub mod friend;
pub mod group;
pub mod member;

#[derive(Clone)]
pub enum Contact {
    Friend(Friend),
    Group(Group),
    Member(Member),
    Stranger,
}

impl Contact {
    pub async fn upload_image(&self, img: Vec<u8>) -> Result<Image, AtriError> {
        match self {
            Self::Friend(f) => f.upload_image(img).await,
            Self::Group(g) => g.upload_image(img).await,
            Self::Member(_) => Err(AtriError::NotSupported),
            Self::Stranger => todo!(),
        }
    }

    pub async fn send_message<M: Into<MessageChain>>(
        &self,
        chain: M,
    ) -> Result<MessageReceipt, AtriError> {
        match self {
            Self::Friend(f) => f.send_message(chain).await,
            Self::Group(g) => g.send_message(chain).await,
            Self::Member(_) => todo!(),
            Self::Stranger => todo!(),
        }
    }
}

pub trait HasSubject {
    fn subject(&self) -> Contact;
}
