use std::path::PathBuf;
use crate::loader::{get_plugin_handle, get_plugin_manager_vtb};

pub fn workspace() -> PathBuf {
    let s: String = (get_plugin_manager_vtb().env_get_workspace)(get_plugin_handle()).into();

    PathBuf::from(s)
}