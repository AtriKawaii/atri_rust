use crate::loader::{get_plugin_manager, get_vtb};
use crate::runtime::JoinHandle;
use atri_ffi::future::FFIFuture;
use atri_ffi::Managed;
use std::future::Future;

pub struct PluginRuntime;

impl PluginRuntime {
    /// 使用插件共享协程执行器执行协程，返回JoinHandle
    ///
    /// 注意：返回值会经过一次Box装箱拆箱，请避免返回过大的值
    pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
    where
        F: Future,
        F: Send + 'static,
        F::Output: Send + 'static,
    {
        let ffi = FFIFuture::from_static(async move {
            let value: F::Output = future.await;

            Managed::from_value(value)
        });

        let f = (get_vtb().plugin_manager_spawn)(get_plugin_manager(), ffi);
        let handle = JoinHandle::<F::Output>::from(f);
        handle
    }

    /// 阻塞当前线程执行协程，并返回Future的返回值
    ///
    /// 注意：返回值会经过一次Box装箱拆箱，请避免返回过大的值
    pub fn block_on<F>(future: F) -> F::Output
    where
        F: Future,
    {
        let ffi = FFIFuture::from(async move {
            let value: F::Output = future.await;

            Managed::from_value(value)
        });
        let managed = (get_vtb().plugin_manager_block_on)(get_plugin_manager(), ffi);
        unsafe { managed.into_value() }
    }
}
