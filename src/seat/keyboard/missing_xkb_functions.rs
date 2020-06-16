use xkbcommon_sys as ffi;
#[derive(Debug)]
pub struct KeyMapError;

impl std::fmt::Display for KeyMapError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error getting keymap")
    }
}

impl std::error::Error for KeyMapError {}

pub fn keymap_from_buffer(
    context: &xkb::Context,
    buffer: &[u8],
) -> Result<xkb::Keymap, KeyMapError> {
    unsafe {
        let ptr = ffi::xkb_keymap_new_from_buffer(
            context.as_ptr(),
            buffer.as_ptr().cast(),
            buffer.len(),
            ffi::XKB_KEYMAP_FORMAT_TEXT_v1,
            ffi::XKB_KEYMAP_COMPILE_NO_FLAGS,
        );
        if ptr.is_null() {
            Err(KeyMapError)
        } else {
            Ok(xkb::Keymap::from_ptr(ptr))
        }
    }
}
