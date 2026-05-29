mod composer;
mod hangul;
mod layout;

pub use composer::{Composer, InputEffect};
pub use layout::{JamoRole, KeyEntry, KeyMapping, Layout, LayoutError};

pub fn crate_name() -> &'static str {
    "yido-core"
}
