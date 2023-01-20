use crate::error;
use crate::event::{Event, FromEvent};
use crate::loader::get_vtb;
use atri_ffi::closure::FFIFn;
use atri_ffi::ffi::FFIEvent;
use atri_ffi::future::FFIFuture;
use atri_ffi::Managed;
use std::future::Future;
use std::time::Duration;

pub struct Listener;

#[repr(u8)]
#[derive(Copy, Clone, Default)]
pub enum Priority {
    Top = 0,
    High = 1,
    #[default]
    Middle = 2,
    Low = 3,
    Base = 4,
}

impl Listener {
    #[inline]
    pub fn listening_on<E, F, Fu>(handler: F) -> ListenerGuard
    where
        F: Fn(E) -> Fu,
        F: Send + 'static,
        Fu: Future<Output = bool>,
        Fu: Send + 'static,
        E: FromEvent,
    {
        ListenerBuilder::listening_on(handler).start()
    }

    #[inline]
    pub fn listening_on_always<E, F, Fu>(handler: F) -> ListenerGuard
    where
        F: Fn(E) -> Fu,
        F: Send + 'static,
        Fu: Future<Output = ()>,
        Fu: Send + 'static,
        E: FromEvent,
    {
        ListenerBuilder::listening_on_always(handler).start()
    }

    #[inline]
    pub async fn next_event<E, F>(timeout: Duration, filter: F) -> Option<E>
    where
        E: FromEvent,
        E: Send + 'static,
        F: Fn(&E) -> bool,
    {
        Self::next_event_with_priority(timeout, filter, Priority::Middle).await
    }

    pub async fn next_event_with_priority<E, F>(
        timeout: Duration,
        filter: F,
        priority: Priority,
    ) -> Option<E>
    where
        E: FromEvent,
        E: Send + 'static,
        F: Fn(&E) -> bool,
    {
        let ffi = crate::runtime::spawn((get_vtb().listener_next_event_with_priority)(
            timeout.as_millis() as u64,
            FFIFn::from(|ffi| {
                let event = Event::from_ffi(ffi);

                E::from_event(event).as_ref().map(&filter).unwrap_or(false)
            }),
            priority as u8,
        ))
        .await
        .unwrap();

        Option::<FFIEvent>::from(ffi).and_then(|ffi| {
            let event = Event::from_ffi(ffi);
            E::from_event(event)
        })
    }
}

pub struct ListenerBuilder {
    concurrent: bool,
    handler: FFIFn<FFIEvent, FFIFuture<bool>>,
    priority: Priority,
}

impl ListenerBuilder {
    fn new<F, Fu>(handler: F) -> Self
    where
        F: Fn(Event) -> Fu,
        F: Send + 'static,
        Fu: Future<Output = bool>,
        Fu: Send + 'static,
    {
        let f = FFIFn::from_static(move |ffi| {
            let fu = handler(Event::from_ffi(ffi));
            FFIFuture::from_static(async move {
                crate::runtime::spawn(fu).await.unwrap_or_else(|e| {
                    error!("监听器发生预料之外的错误, 停止监听: {}", e);
                    false
                })
            })
        });

        Self {
            concurrent: true,
            handler: f,
            priority: Priority::Middle,
        }
    }

    #[inline]
    fn new_always<F, Fu>(handler: F) -> Self
    where
        F: Fn(Event) -> Fu,
        F: Send + 'static,
        Fu: Future<Output = ()>,
        Fu: Send + 'static,
    {
        Self::new(move |e: Event| {
            let fu = handler(e);

            async move {
                fu.await;
                true
            }
        })
    }

    #[inline]
    pub fn listening_on<E, F, Fu>(handler: F) -> Self
    where
        F: Fn(E) -> Fu,
        F: Send + 'static,
        Fu: Future<Output = bool>,
        Fu: Send + 'static,
        E: FromEvent,
    {
        Self::new(move |e: Event| {
            let fu = E::from_event(e).and_then(|e| Some(handler(e)));

            async move {
                if let Some(fu) = fu {
                    fu.await
                } else {
                    true
                }
            }
        })
    }

    #[inline]
    pub fn listening_on_always<E, F, Fu>(handler: F) -> Self
    where
        F: Fn(E) -> Fu,
        F: Send + 'static,
        Fu: Future<Output = ()>,
        Fu: Send + 'static,
        E: FromEvent,
    {
        Self::new_always(move |e: Event| {
            let fu = E::from_event(e).and_then(|e| Some(handler(e)));

            async move {
                if let Some(fu) = fu {
                    fu.await;
                }
            }
        })
    }

    #[inline]
    pub fn concurrent(mut self, is: bool) -> Self {
        self.concurrent = is;
        self
    }

    #[inline]
    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn start(self) -> ListenerGuard {
        let ma = (get_vtb().new_listener)(self.concurrent, self.handler, self.priority as u8);
        ListenerGuard(ma)
    }
}

#[must_use = "if unused the Listener will immediately close"]
pub struct ListenerGuard(Managed);

impl ListenerGuard {
    pub fn close(self) {
        drop(self);
    }
}
