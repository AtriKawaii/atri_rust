use crate::bot::Bot;
use crate::contact::friend::Friend;
use crate::contact::group::Group;
use crate::contact::member::Member;
use crate::listener::Listener;
use crate::loader::get_plugin_manager_vtb;
use crate::message::MessageChain;
use atri_ffi::ffi::{FFIEvent, ForFFI};
use atri_ffi::ManagedCloneable;
use std::ops::Deref;
use std::time::Duration;

#[derive(Clone)]
pub enum Event {
    BotLogin(BotLoginEvent),
    GroupMessage(GroupMessageEvent),
    FriendMessage(FriendMessageEvent),
    Unknown(EventInner),
}

impl Event {
    pub fn from_ffi(ffi: FFIEvent) -> Self {
        let (t, intercepted, m) = ffi.get();
        let inner = EventInner {
            intercepted,
            event: m,
        };

        match t {
            0 => Self::BotLogin(BotLoginEvent(inner)),
            1 => Self::GroupMessage(GroupMessageEvent(inner)),
            2 => Self::FriendMessage(FriendMessageEvent(inner)),
            _ => Self::Unknown(inner),
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
        (get_plugin_manager_vtb().event_intercept)(self.intercepted);
    }

    pub fn is_intercepted(&self) -> bool {
        (get_plugin_manager_vtb().event_is_intercepted)(self.intercepted)
    }
}

unsafe impl Send for EventInner {}

unsafe impl Sync for EventInner {}

#[derive(Clone)]
pub struct BotLoginEvent(EventInner);

#[derive(Clone)]
pub struct GroupMessageEvent(EventInner);

impl GroupMessageEvent {
    pub fn group(&self) -> Group {
        let ma = (get_plugin_manager_vtb().group_message_event_get_group)(self.0.event.pointer);
        Group(ma)
    }

    pub fn bot(&self) -> Bot {
        self.group().bot()
    }

    pub fn sender(&self) -> Member {
        let ffi = (get_plugin_manager_vtb().group_message_event_get_sender)(self.event.pointer);

        Member::from_ffi(ffi)
    }

    pub fn message(&self) -> MessageChain {
        let ffi = (get_plugin_manager_vtb().group_message_event_get_message)(self.0.event.pointer);
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
    pub fn friend(&self) -> Friend {
        let ma = (get_plugin_manager_vtb().friend_message_event_get_friend)(self.event.pointer);
        Friend(ma)
    }

    pub fn bot(&self) -> Bot {
        self.friend().bot()
    }

    pub fn message(&self) -> MessageChain {
        let ffi = (get_plugin_manager_vtb().friend_message_event_get_message)(self.event.pointer);
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

event_inner_impl! {
    BotLoginEvent
    GroupMessageEvent
    FriendMessageEvent
}
