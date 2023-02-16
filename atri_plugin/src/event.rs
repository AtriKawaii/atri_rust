use crate::client::Client;
use crate::contact::friend::Friend;
use crate::contact::group::Group;
use crate::contact::member::Member;
use crate::listener::Listener;
use crate::loader::get_vtb;
use crate::message::MessageChain;
use crate::warn;
use atri_ffi::ffi::{FFIEvent, ForFFI};
use atri_ffi::ManagedCloneable;
use std::ops::Deref;
use std::time::Duration;

#[derive(Clone)]
pub enum Event {
    ClientLogin(ClientLoginEvent),
    GroupMessage(GroupMessageEvent),
    FriendMessage(FriendMessageEvent),
    NewFriend(NewFriendEvent),
    DeleteFriend(DeleteFriendEvent),
    Unknown { raw_tag: u8, inner: EventInner },
}

impl Event {
    pub fn from_ffi(ffi: FFIEvent) -> Self {
        macro_rules! event_matches {
            (type: $t:expr, inner: $in:expr; $($nr:expr => ($e:ident,$inner:ident));* $(;)?) => {
                match $t {
                    $($nr => Self::$e($inner($in)),)*
                    or => {
                        if or != 255 {
                           warn!("接受了一个未知事件, tag={}", or);
                        }

                        Self::Unknown { raw_tag: or, inner: $in }
                    }
                }
            };
        }

        let (t, intercepted, m) = ffi.get();
        let inner = EventInner {
            intercepted,
            event: m,
        };

        event_matches! {
            type: t, inner: inner;
            0 => (ClientLogin, ClientLoginEvent);
            1 => (GroupMessage, GroupMessageEvent);
            2 => (FriendMessage, FriendMessageEvent);
            3 => (NewFriend, NewFriendEvent);
            4 => (DeleteFriend, DeleteFriendEvent);
        }
    }
}

impl FromEvent for Event {
    fn from_event(e: Event) -> Option<Self> {
        Some(e)
    }
}

#[derive(Clone)]
pub struct EventInner {
    intercepted: *const (), // owned by event
    event: ManagedCloneable,
}

impl EventInner {
    pub fn intercept(&self) {
        (get_vtb().event_intercept)(self.intercepted);
    }

    pub fn is_intercepted(&self) -> bool {
        (get_vtb().event_is_intercepted)(self.intercepted)
    }
}

unsafe impl Send for EventInner {}

unsafe impl Sync for EventInner {}

#[derive(Clone)]
pub struct ClientLoginEvent(EventInner);

#[derive(Clone)]
pub struct GroupMessageEvent(EventInner);

impl GroupMessageEvent {
    pub fn group(&self) -> &Group {
        let phandle = (get_vtb().group_message_event_get_group)(self.0.event.pointer);
        unsafe { std::mem::transmute(phandle) }
    }

    pub fn client(&self) -> Client {
        self.group().client()
    }

    pub fn sender(&self) -> Member {
        let ffi = (get_vtb().group_message_event_get_sender)(self.event.pointer);

        Member::from_ffi(ffi)
    }

    pub fn message(&self) -> MessageChain {
        let ffi = (get_vtb().group_message_event_get_message)(self.0.event.pointer);
        MessageChain::from_ffi(ffi)
    }

    pub async fn next<F>(&self, timeout: Duration, filter: F) -> Option<Self>
    where
        F: Fn(&Self) -> bool,
        F: Send + 'static,
    {
        let group_id = self.group().id();
        let sender_id = self.sender().id();
        Listener::next_event(timeout, move |e: &Self| {
            if e.group().id() != group_id || e.sender().id() != sender_id {
                return false;
            }

            filter(e)
        })
        .await
    }
}

impl FromEvent for GroupMessageEvent {
    fn from_event(e: Event) -> Option<Self> {
        if let Event::GroupMessage(e) = e {
            Some(e)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct FriendMessageEvent(EventInner);

impl FriendMessageEvent {
    pub fn friend(&self) -> &Friend {
        let phandle = (get_vtb().friend_message_event_get_friend)(self.event.pointer);
        unsafe { std::mem::transmute(phandle) }
    }

    pub fn client(&self) -> Client {
        self.friend().client()
    }

    pub fn message(&self) -> MessageChain {
        let ffi = (get_vtb().friend_message_event_get_message)(self.event.pointer);
        MessageChain::from_ffi(ffi)
    }

    pub async fn next<F>(&self, timeout: Duration, filter: F) -> Option<Self>
    where
        F: Fn(&Self) -> bool,
        F: Send + 'static,
    {
        let friend_id = self.friend().id();
        Listener::next_event(timeout, move |e: &Self| {
            if e.friend().id() != friend_id {
                return false;
            }

            filter(e)
        })
        .await
    }
}

#[derive(Clone)]
pub struct NewFriendEvent(EventInner);

impl NewFriendEvent {}

#[derive(Clone)]
pub struct DeleteFriendEvent(EventInner);

impl DeleteFriendEvent {}

impl FromEvent for FriendMessageEvent {
    fn from_event(e: Event) -> Option<Self> {
        if let Event::FriendMessage(e) = e {
            Some(e)
        } else {
            None
        }
    }
}

pub trait FromEvent: Sized {
    fn from_event(e: Event) -> Option<Self>;
}

macro_rules! event_inner_impl {
    ($($t:ty)*) => {
        $(
        impl Deref for $t {
            type Target = EventInner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        )*
    };
}

pub enum MessageEvent {
    Friend(FriendMessageEvent),
    Group(GroupMessageEvent),
}

impl FromEvent for MessageEvent {
    fn from_event(e: Event) -> Option<Self> {
        let e = match e {
            Event::FriendMessage(e) => Self::Friend(e),
            Event::GroupMessage(e) => Self::Group(e),
            _ => return None,
        };

        Some(e)
    }
}

event_inner_impl! {
    ClientLoginEvent
    GroupMessageEvent
    FriendMessageEvent
}
