use crate::event::{Event, FromEvent};
use crate::loader::get_plugin_manager_vtb;
use atri_ffi::closure::FFIFn;
use atri_ffi::future::FFIFuture;
use atri_ffi::Managed;
use std::future::Future;

pub struct Listener;

impl Listener {
    pub fn new<F, Fu>(handler: F) -> ListenerGuard
    where
        F: Fn(Event) -> Fu,
        F: Send + 'static,
        Fu: Future<Output = bool>,
        Fu: Send + 'static,
    {
        let f = FFIFn::from_static(move |ffi| {
            let fu = handler(Event::from_ffi(ffi));

            FFIFuture::from_static(async move { crate::runtime::spawn(fu).await.unwrap_or(false) })
        });
        let ma = (get_plugin_manager_vtb().new_listener)(f);
        ListenerGuard(ma)
    }

    fn new_always<F, Fu>(handler: F) -> ListenerGuard
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

    pub fn listening_on<E, F, Fu>(handler: F) -> ListenerGuard
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

    pub fn listening_on_always<E, F, Fu>(handler: F) -> ListenerGuard
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
}

pub struct ListenerGuard(Managed);
