use crate::contact::friend::Friend;
use crate::contact::group::Group;
use crate::loader::get_vtb;
use atri_ffi::ManagedCloneable;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct Client(pub(crate) ManagedCloneable);

impl Client {
    pub fn id(&self) -> i64 {
        (get_vtb().client_get_id)(self.0.pointer)
    }

    pub fn nickname(&self) -> String {
        let rs = (get_vtb().client_get_nickname)(self.0.pointer);

        rs.into()
    }

    pub fn list() -> Vec<Client> {
        let raw = (get_vtb().client_get_list)();

        raw.into_vec().into_iter().map(Client).collect()
    }

    pub fn find(id: i64) -> Option<Self> {
        let ma = (get_vtb().find_client)(id);

        if ma.pointer.is_null() {
            None
        } else {
            Some(Self(ma))
        }
    }

    pub fn find_group(&self, id: i64) -> Option<Group> {
        let ma = (get_vtb().client_find_group)(self.0.pointer, id);

        if ma.pointer.is_null() {
            None
        } else {
            Some(Group(ma))
        }
    }

    pub fn find_friend(&self, id: i64) -> Option<Friend> {
        let ma = (get_vtb().client_find_friend)(self.0.pointer, id);

        if ma.pointer.is_null() {
            None
        } else {
            Some(Friend(ma))
        }
    }

    pub fn groups(&self) -> Vec<Group> {
        let ma = (get_vtb().client_get_groups)(self.0.pointer);
        ma.into_vec().into_iter().map(Group).collect()
    }

    pub fn friends(&self) -> Vec<Friend> {
        let ma = (get_vtb().client_get_friends)(self.0.pointer);
        ma.into_vec().into_iter().map(Friend).collect()
    }
}

impl Display for Client {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Client({})", self.id())
    }
}
