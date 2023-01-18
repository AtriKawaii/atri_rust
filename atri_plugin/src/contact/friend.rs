use crate::client::Client;
use crate::error::AtriError;
use crate::loader::get_vtb;
use crate::message::image::Image;
use crate::message::meta::MessageReceipt;
use crate::message::MessageChain;
use atri_ffi::ffi::ForFFI;
use atri_ffi::{ManagedCloneable, RustVec};
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct Friend(pub(crate) ManagedCloneable);

impl Friend {
    pub fn id(&self) -> i64 {
        (get_vtb().friend_get_id)(self.0.pointer)
    }

    pub fn nickname(&self) -> &str {
        let rs = (get_vtb().friend_get_nickname)(self.0.pointer);

        rs.as_str()
    }

    pub fn client(&self) -> Client {
        let ma = (get_vtb().friend_get_client)(self.0.pointer);
        Client(ma)
    }

    pub async fn send_message<M: Into<MessageChain>>(
        &self,
        chain: M,
    ) -> Result<MessageReceipt, AtriError> {
        let fu = {
            let ffi = chain.into().into_ffi();
            (get_vtb().friend_send_message)(self.0.pointer, ffi)
        };

        let result = Result::from(crate::runtime::spawn(fu).await.unwrap());
        match result {
            Ok(ffi) => Ok(MessageReceipt::from_ffi(ffi)),
            Err(s) => Err(AtriError::ClientError(s)),
        }
    }

    pub async fn upload_image(&self, img: Vec<u8>) -> Result<Image, AtriError> {
        let fu = { (get_vtb().friend_upload_image)(self.0.pointer, RustVec::from(img)) };
        let result = crate::runtime::spawn(fu).await.unwrap();

        match Result::from(result) {
            Ok(ma) => Ok(Image(ma)),
            Err(s) => Err(AtriError::ClientError(s)),
        }
    }
}

impl Display for Friend {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Friend({})", self.id())
    }
}
