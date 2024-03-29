use crate::client::Client;
use crate::contact::group::Group;
use crate::error::AtriError;
use crate::loader::get_vtb;
use atri_ffi::contact::FFIMember;
use atri_ffi::{ManagedCloneable, RustStr};
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub enum Member {
    Named(NamedMember),
    Anonymous(AnonymousMember),
}

impl Member {
    pub fn id(&self) -> i64 {
        match self {
            Self::Named(named) => named.id(),
            Self::Anonymous(_ano) => 80000000,
        }
    }
}

impl Member {
    pub(crate) fn from_ffi(ffi: FFIMember) -> Self {
        if ffi.is_named {
            Self::Named(NamedMember(ffi.inner))
        } else {
            Self::Anonymous(AnonymousMember(ffi.inner))
        }
    }
}

#[derive(Clone)]
pub struct NamedMember(pub(crate) ManagedCloneable);

impl NamedMember {
    pub fn id(&self) -> i64 {
        (get_vtb().named_member_get_id)(self.0.pointer)
    }

    pub fn nickname(&self) -> &str {
        let rs = (get_vtb().named_member_get_nickname)(self.0.pointer);

        rs.as_str()
    }

    pub fn card_name(&self) -> &str {
        let rs = (get_vtb().named_member_get_card_name)(self.0.pointer);

        rs.as_str()
    }

    pub fn group(&self) -> Group {
        let handle = (get_vtb().named_member_get_group)(self.0.pointer);
        Group(handle)
    }

    pub fn client(&self) -> Client {
        self.group().client()
    }

    pub async fn change_card_name(&self, card_name: &str) -> Result<(), AtriError> {
        let rs = RustStr::from(card_name);

        let fu = (get_vtb().named_member_change_card_name)(self.0.pointer, rs);

        let result = crate::runtime::spawn(fu).await.unwrap();
        Result::from(result).map_err(AtriError::ClientError)
    }
}

impl Display for NamedMember {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "NamedMember({})", self.id())
    }
}

#[derive(Clone)]
pub struct AnonymousMember(ManagedCloneable);
