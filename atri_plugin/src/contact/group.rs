use crate::client::Client;
use crate::contact::member::NamedMember;
use crate::error::{AtriError, AtriResult};
use crate::loader::get_plugin_manager_vtb;
use crate::message::image::Image;
use crate::message::meta::MessageReceipt;
use crate::message::MessageChain;
use atri_ffi::error::FFIResult;
use atri_ffi::ffi::ForFFI;
use atri_ffi::message::FFIMessageChain;
use atri_ffi::{ManagedCloneable, RustStr};
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct Group(pub(crate) ManagedCloneable);

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Group {
    pub fn id(&self) -> i64 {
        (get_plugin_manager_vtb().group_get_id)(self.0.pointer)
    }

    pub fn client(&self) -> Client {
        let ma = (get_plugin_manager_vtb().group_get_client)(self.0.pointer);
        Client(ma)
    }

    pub fn name(&self) -> &str {
        let rs = (get_plugin_manager_vtb().group_get_name)(self.0.pointer);
        // Safety: this slice should live as long as self(Group)
        rs.as_str()
    }

    pub async fn members(&self) -> Vec<NamedMember> {
        let fu = { (get_plugin_manager_vtb().group_get_members)(self.0.pointer) };
        let ma = crate::runtime::spawn(fu).await.unwrap().into_vec();
        ma.into_iter().map(NamedMember).collect()
    }

    pub fn find_member(&self, id: i64) -> Option<NamedMember> {
        let ma = (get_plugin_manager_vtb().group_find_member)(self.0.pointer, id);

        if ma.pointer.is_null() {
            None
        } else {
            Some(NamedMember(ma))
        }
    }

    pub async fn get_named_member(&self, id: i64) -> Option<NamedMember> {
        let fu = { (get_plugin_manager_vtb().group_get_named_member)(self.0.pointer, id) };
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
            (get_plugin_manager_vtb().group_send_message)(self.0.pointer, ffi)
        };

        let res = crate::runtime::spawn(fu).await.unwrap();
        match Result::from(res) {
            Ok(ffi) => Ok(MessageReceipt::from_ffi(ffi)),
            Err(s) => Err(AtriError::ClientError(s)),
        }
    }

    pub async fn upload_image(&self, image: Vec<u8>) -> AtriResult<Image> {
        let fu = { (get_plugin_manager_vtb().group_upload_image)(self.0.pointer, image.into()) };

        let result = crate::runtime::spawn(fu).await.unwrap();
        match Result::from(result) {
            Ok(ma) => Ok(Image(ma)),
            Err(e) => Err(AtriError::ClientError(e)),
        }
    }

    pub async fn change_name(&self, name: &str) -> AtriResult<()> {
        let rs = RustStr::from(name);
        let fu = { (get_plugin_manager_vtb().group_change_name)(self.0.pointer, rs) };
        let result: FFIResult<()> = crate::runtime::spawn(fu).await.unwrap();

        Result::from(result).map_err(|s| AtriError::ClientError(s))
    }

    pub async fn quit(&self) -> bool {
        crate::runtime::spawn((get_plugin_manager_vtb().group_quit)(self.0.pointer))
            .await
            .unwrap()
    }
}

impl Display for Group {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Group({})", self.id())
    }
}
