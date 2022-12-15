use atri_ffi::plugin::PluginVTable;
use atri_ffi::RustStr;

pub use atri_macros::plugin;

pub mod client;
pub mod contact;
pub mod env;
pub mod error;
pub mod event;
pub mod listener;
pub mod loader;
pub mod log;
pub mod message;
pub mod runtime;

pub use atri_ffi::plugin::PluginInstance;

pub trait Plugin
where
    Self: Sized,
{
    /// 构造插件实例
    fn new() -> Self;

    /// 插件启用
    ///
    /// 若`should_drop`为`true`, 则再次启用插件前会先构造插件实例
    fn enable(&mut self);

    /// 插件禁用
    fn disable(&mut self) {
        // default impl: nop
    }

    /// 是否应该在插件被禁用后销毁插件实例
    ///
    /// 若为`false`，则插件只会在卸载时销毁实例
    fn should_drop() -> bool {
        true
    }
}

#[doc(hidden)]
/// 从已实现Plugin的结构体获取一个标准的PluginInstance
pub fn __get_instance<P: Plugin>(name: &str) -> PluginInstance {
    extern "C" fn _new<P: Plugin>() -> *mut () {
        let b = Box::new(P::new());
        Box::into_raw(b) as *mut ()
    }

    extern "C" fn _enable<P: Plugin>(ptr: *mut ()) {
        // Safety: Plugin is pinned by box
        let p = unsafe { &mut *(ptr as *mut P) };
        p.enable();
    }

    extern "C" fn _disable<P: Plugin>(ptr: *mut ()) {
        // Safety: Plugin is pinned by box
        let p = unsafe { &mut *(ptr as *mut P) };
        p.disable();
    }

    extern "C" fn _drop<T>(ptr: *mut ()) {
        drop(unsafe {
            Box::from_raw(ptr.cast::<T>())
        })
    }

    let should_drop = P::should_drop();

    let vtb = PluginVTable {
        new: _new::<P>,
        enable: _enable::<P>,
        disable: _disable::<P>,
        drop: _drop::<P>
    };

    PluginInstance {
        should_drop,
        vtb,
        abi_ver: atri_ffi::plugin::abi_version(),
        name: RustStr::from(name),
    }
}
