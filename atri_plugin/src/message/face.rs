#[derive(Clone, Debug)]
pub struct Face {
    pub index: i32,
    pub name: String,
}

mod ffi {
    use crate::message::face::Face;
    use atri_ffi::ffi::ForFFI;
    use atri_ffi::message::FFIFace;

    impl ForFFI for Face {
        type FFIValue = FFIFace;

        fn into_ffi(self) -> Self::FFIValue {
            let Self { index, name } = self;

            FFIFace {
                index,
                name: name.into(),
            }
        }

        fn from_ffi(FFIFace { index, name }: Self::FFIValue) -> Self {
            Self {
                index,
                name: name.into(),
            }
        }
    }
}
