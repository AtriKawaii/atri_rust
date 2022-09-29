use crate::error::{AtriError, AtriResult};
use crate::runtime::manager::PluginRuntime;
use atri_ffi::error::FFIResult;
use atri_ffi::future::FFIFuture;
use atri_ffi::Managed;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

mod manager;

pub struct JoinHandle<T> {
    handle: FFIFuture<FFIResult<Managed>>,
    _mark: PhantomData<T>,
}

impl<T> JoinHandle<T> {
    pub fn from(f: FFIFuture<FFIResult<Managed>>) -> Self {
        Self {
            handle: f,
            _mark: PhantomData,
        }
    }
}

impl<T> Unpin for JoinHandle<T> {}

unsafe impl<T: Send> Send for JoinHandle<T> {}
unsafe impl<T: Send> Sync for JoinHandle<T> {}

impl<T> Future for JoinHandle<T> {
    type Output = AtriResult<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let pin = unsafe { Pin::new_unchecked(&mut self.handle) };

        match pin.poll(cx) {
            Poll::Ready(ffi) => {
                let result = match Result::from(ffi) {
                    Ok(val) => Ok(val.into_value()),
                    Err(s) => Err(AtriError::JoinError(s)),
                };
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// 使用插件共享协程执行器执行协程，返回JoinHandle
///
/// 注意：返回值会经过一次Box装箱拆箱，请避免返回过大的值
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future,
    F: Send + 'static,
    F::Output: Send + 'static,
{
    PluginRuntime::spawn(future)
}

/// 阻塞当前线程执行协程，并返回Future的返回值
///
/// 注意：返回值会经过一次Box装箱拆箱，请避免返回过大的值
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    PluginRuntime::block_on(future)
}
