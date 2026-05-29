mod layout;

pub use layout::{JamoRole, KeyEntry, KeyMapping, Layout, LayoutError};

pub fn crate_name() -> &'static str {
    "yido-core"
}
