use atri_ffi::closure::FFIFn;
use atri_ffi::contact::FFIMember;
use atri_ffi::error::FFIResult;
use atri_ffi::ffi::{AtriManager, FFIEvent};
use atri_ffi::future::FFIFuture;
use atri_ffi::message::forward::FFIForwardNode;
use atri_ffi::message::{FFIMessageChain, FFIMessageReceipt};
use atri_ffi::{
    FFIOption, Handle, Managed, ManagedCloneable, PHandle, RustStr, RustString, RustVec,
};
use std::mem::MaybeUninit;

pub struct AtriVTable {
    pub plugin_manager_spawn:
        extern "C" fn(manager: *const (), FFIFuture<Managed>) -> FFIFuture<FFIResult<Managed>>,
    pub plugin_manager_block_on: extern "C" fn(manager: *const (), FFIFuture<Managed>) -> Managed,

    pub new_listener: extern "C" fn(bool, FFIFn<FFIEvent, FFIFuture<bool>>, u8) -> Managed,
    pub listener_next_event_with_priority: extern "C" fn(
        millis: u64,
        filter: FFIFn<FFIEvent, bool>,
        priority: u8,
    ) -> FFIFuture<FFIOption<FFIEvent>>,

    pub event_intercept: extern "C" fn(*const ()),
    pub event_is_intercepted: extern "C" fn(*const ()) -> bool,

    pub client_get_id: extern "C" fn(Handle) -> i64,
    pub client_get_nickname: extern "C" fn(Handle) -> RustString,
    pub client_get_list: extern "C" fn() -> RustVec<Handle>,
    pub find_client: extern "C" fn(i64) -> Handle,
    pub client_find_group: extern "C" fn(Handle, i64) -> Handle,
    pub client_find_friend: extern "C" fn(Handle, i64) -> Handle,
    pub client_get_groups: extern "C" fn(Handle) -> RustVec<Handle>,
    pub client_get_friends: extern "C" fn(Handle) -> RustVec<Handle>,
    pub client_clone: extern "C" fn(Handle) -> Handle,
    pub client_drop: extern "C" fn(Handle),

    pub group_message_event_get_group: extern "C" fn(event: *const ()) -> PHandle,
    pub group_message_event_get_message: extern "C" fn(event: *const ()) -> FFIMessageChain,
    pub group_message_event_get_sender: extern "C" fn(event: *const ()) -> FFIMember,

    pub group_get_id: extern "C" fn(Handle) -> i64,
    pub group_get_name: extern "C" fn(Handle) -> RustStr,
    pub group_get_client: extern "C" fn(Handle) -> PHandle,
    pub group_get_members: extern "C" fn(Handle) -> FFIFuture<RustVec<ManagedCloneable>>,
    pub group_find_member: extern "C" fn(Handle, i64) -> FFIFuture<ManagedCloneable>,
    pub group_send_message: extern "C" fn(
        group: Handle,
        chain: FFIMessageChain,
    ) -> FFIFuture<FFIResult<FFIMessageReceipt>>,
    pub group_upload_image:
        extern "C" fn(group: Handle, data: RustVec<u8>) -> FFIFuture<FFIResult<ManagedCloneable>>,
    pub group_quit: extern "C" fn(group: Handle) -> FFIFuture<bool>,
    pub group_change_name: extern "C" fn(group: Handle, name: RustStr) -> FFIFuture<FFIResult<()>>,
    pub group_send_forward_message: extern "C" fn(
        group: *const (),
        msg: RustVec<FFIForwardNode>,
    ) -> FFIFuture<FFIResult<FFIMessageReceipt>>,
    pub group_invite: extern "C" fn(group: Handle, id: i64) -> FFIFuture<FFIResult<()>>,
    pub group_clone: extern "C" fn(Handle) -> Handle,
    pub group_drop: extern "C" fn(Handle),

    pub friend_message_event_get_friend: extern "C" fn(event: *const ()) -> PHandle,
    pub friend_message_event_get_message: extern "C" fn(event: *const ()) -> FFIMessageChain,
    pub friend_get_id: extern "C" fn(Handle) -> i64,
    pub friend_get_nickname: extern "C" fn(Handle) -> RustStr,
    pub friend_get_client: extern "C" fn(Handle) -> PHandle,
    pub friend_send_message: extern "C" fn(
        friend: Handle,
        chain: FFIMessageChain,
    ) -> FFIFuture<FFIResult<FFIMessageReceipt>>,
    pub friend_upload_image:
        extern "C" fn(friend: Handle, img: RustVec<u8>) -> FFIFuture<FFIResult<ManagedCloneable>>,
    pub friend_clone: extern "C" fn(Handle) -> Handle,
    pub friend_drop: extern "C" fn(Handle),

    pub named_member_get_id: extern "C" fn(named: *const ()) -> i64,
    pub named_member_get_nickname: extern "C" fn(named: *const ()) -> RustStr,
    pub named_member_get_card_name: extern "C" fn(named: *const ()) -> RustStr,
    pub named_member_get_group: extern "C" fn(named: *const ()) -> PHandle,
    pub named_member_change_card_name:
        extern "C" fn(named: *const (), card: RustStr) -> FFIFuture<FFIResult<()>>,

    pub image_get_id: extern "C" fn(img: *const ()) -> RustStr,
    // flash
    pub image_get_url: extern "C" fn(img: *const ()) -> RustString,

    pub log: extern "C" fn(handle: usize, manager: *const (), level: u8, log: RustStr),

    pub env_get_workspace: extern "C" fn(handle: usize, manager: *const ()) -> RustString,

    pub message_chain_to_json: extern "C" fn(chain: FFIMessageChain) -> RustString,
    pub message_chain_from_json: extern "C" fn(json: RustStr) -> FFIResult<FFIMessageChain>,
}

static mut ATRI_MANAGER: MaybeUninit<AtriManager> = MaybeUninit::uninit();

static mut ATRI_VTABLE: MaybeUninit<AtriVTable> = MaybeUninit::uninit();

/// Safety: This function will be called by the plugin manager once
#[no_mangle]
unsafe extern "C" fn atri_manager_init(manager: AtriManager) {
    let get_fun = manager.get_fun;

    ATRI_MANAGER.write(manager);

    macro_rules! vtb {
        (get_fun: $fun:expr; $($field:ident => $sig:expr),* $(,)?) => {
            AtriVTable {
                $($field: std::mem::transmute(($fun)($sig)),)*
            }
        };
    }

    let vtable = vtb! {
        get_fun: get_fun;
        plugin_manager_spawn => 0,
        plugin_manager_block_on => 1,

        new_listener => 100,
        listener_next_event_with_priority => 101,

        event_intercept => 200,
        event_is_intercepted => 201,

        client_get_id => 300,
        client_get_nickname => 301,
        client_get_list => 302,
        find_client => 303,
        client_find_group => 304,
        client_find_friend => 305,
        client_get_groups => 306,
        client_get_friends => 307,
        client_clone => 320,
        client_drop => 321,

        group_get_id => 400,
        group_get_name => 401,
        group_get_client => 402,
        group_get_members => 403,
        group_find_member => 404,
        // 405
        group_send_message => 406,
        group_upload_image => 407,
        group_quit => 408,
        group_change_name => 409,
        group_send_forward_message => 410,
        group_invite => 411,

        group_clone => 420,
        group_drop => 421,

        friend_get_id => 500,
        friend_get_nickname => 501,
        friend_get_client => 502,
        friend_send_message => 503,
        friend_upload_image => 504,

        friend_clone => 520,
        friend_drop => 521,

        named_member_get_id => 600,
        named_member_get_nickname => 601,
        named_member_get_card_name => 602,
        named_member_get_group => 603,
        named_member_change_card_name => 604,

        group_message_event_get_group => 10000,
        group_message_event_get_message => 10001,
        group_message_event_get_sender => 10002,

        friend_message_event_get_friend => 10100,
        friend_message_event_get_message => 10101,

        image_get_id => 2000,
        // flash => 2001
        image_get_url => 2002,

        log => 20000,

        env_get_workspace => 30000,

        message_chain_to_json => 30100,
        message_chain_from_json => 30101,
    };

    ATRI_VTABLE.write(vtable);
}

fn get_atri_manager() -> &'static AtriManager {
    unsafe { ATRI_MANAGER.assume_init_ref() }
}

pub(crate) fn get_plugin_manager() -> *const () {
    get_atri_manager().manager_ptr
}

pub(crate) fn get_plugin_handle() -> usize {
    get_atri_manager().handle
}

pub(crate) fn get_vtb() -> &'static AtriVTable {
    unsafe { ATRI_VTABLE.assume_init_ref() }
}
