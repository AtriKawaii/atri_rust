use crate::loader::{get_plugin_handle, get_plugin_manager, get_vtb};
use std::path::PathBuf;

pub fn workspace() -> PathBuf {
    let s: String = (get_vtb().env_get_workspace)(get_plugin_handle(), get_plugin_manager()).into();

    PathBuf::from(s)
}
