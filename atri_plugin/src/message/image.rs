use crate::loader::get_plugin_manager_vtb;
use crate::message::{MessageElement, PushMessage};
use atri_ffi::ManagedCloneable;

#[derive(Clone)]
pub struct Image(pub(crate) ManagedCloneable);

impl Image {
    pub fn id(&self) -> &str {
        let rs = (get_plugin_manager_vtb().image_get_id)(self.0.pointer);
        rs.as_str()
    }

    pub fn url(&self) -> String {
        let rs = (get_plugin_manager_vtb().image_get_url)(self.0.pointer);
        rs.into()
    }
}

impl PushMessage for Image {
    fn push_to(self, v: &mut Vec<MessageElement>) {
        v.push(MessageElement::Image(self));
    }
}
