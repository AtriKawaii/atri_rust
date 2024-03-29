use crate::client::Client;
use crate::contact::member::NamedMember;
use crate::error::{AtriError, AtriResult};
use crate::loader::get_vtb;
use crate::message::forward::ForwardMessage;
use crate::message::image::Image;
use crate::message::meta::MessageReceipt;
use crate::message::MessageChain;
use atri_ffi::error::FFIResult;
use atri_ffi::ffi::ForFFI;
use atri_ffi::message::FFIMessageChain;
use atri_ffi::{Handle, RustStr};
use std::fmt::{Display, Formatter};

pub struct Group(pub(crate) Handle);

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Group {
    pub fn id(&self) -> i64 {
        (get_vtb().group_get_id)(self.0)
    }

    pub fn client(&self) -> Client {
        let handle = (get_vtb().group_get_client)(self.0);
        Client(handle)
    }

    pub fn name(&self) -> &str {
        let rs = (get_vtb().group_get_name)(self.0);
        // Safety: this slice should live as long as self(Group)
        rs.as_str()
    }

    pub async fn members(&self) -> Vec<NamedMember> {
        let fu = { (get_vtb().group_get_members)(self.0) };
        let ma = crate::runtime::spawn(fu).await.unwrap().into_vec();
        ma.into_iter().map(NamedMember).collect()
    }

    pub async fn find_member(&self, id: i64) -> Option<NamedMember> {
        let fu = { (get_vtb().group_find_member)(self.0, id) };

        let ma = crate::runtime::spawn(fu).await.unwrap();

        if ma.pointer.is_null() {
            None
        } else {
            Some(NamedMember(ma))
        }
    }

    pub async fn send_message<M: Into<MessageChain>>(
        &self,
        chain: M,
    ) -> AtriResult<MessageReceipt> {
        let fu = {
            let ffi: FFIMessageChain = chain.into().into_ffi();
            (get_vtb().group_send_message)(self.0, ffi)
        };

        let res = crate::runtime::spawn(fu).await.unwrap();
        Result::from(res)
            .map(MessageReceipt::from_ffi)
            .map_err(AtriError::ClientError)
    }

    pub async fn send_forward_message<M: Into<ForwardMessage>>(
        &self,
        msg: M,
    ) -> AtriResult<MessageReceipt> {
        let fu = {
            let ffi = msg.into().into_ffi();
            (get_vtb().group_send_forward_message)(self.0, ffi)
        };

        let res = crate::runtime::spawn(fu).await.unwrap();
        Result::from(res)
            .map(MessageReceipt::from_ffi)
            .map_err(AtriError::ClientError)
    }

    pub async fn upload_image(&self, image: Vec<u8>) -> AtriResult<Image> {
        let fu = { (get_vtb().group_upload_image)(self.0, image.into()) };

        let result = crate::runtime::spawn(fu).await.unwrap();
        Result::from(result)
            .map(Image)
            .map_err(AtriError::ClientError)
    }

    pub async fn change_name(&self, name: &str) -> AtriResult<()> {
        let rs = RustStr::from(name);
        let fu = { (get_vtb().group_change_name)(self.0, rs) };
        let result: FFIResult<()> = crate::runtime::spawn(fu).await.unwrap();

        Result::from(result).map_err(|s| AtriError::ClientError(s))
    }

    pub async fn quit(&self) -> bool {
        crate::runtime::spawn((get_vtb().group_quit)(self.0))
            .await
            .unwrap()
    }
}

impl Clone for Group {
    fn clone(&self) -> Self {
        Self((get_vtb().group_clone)(self.0))
    }
}

impl Drop for Group {
    fn drop(&mut self) {
        (get_vtb().group_drop)(self.0)
    }
}

unsafe impl Send for Group {}
unsafe impl Sync for Group {}

impl Display for Group {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Group({})", self.id())
    }
}
