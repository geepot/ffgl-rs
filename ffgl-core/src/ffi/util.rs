use core::slice;
use std::ffi::CString;

/// # Safety
///
/// `address` must be a valid pointer to a buffer of at least `max_to_write` bytes.
pub unsafe fn copy_str_to_host_buffer(address: *mut u8, max_to_write: usize, string: &str) {
    let cstr = CString::new(string).unwrap().into_bytes_with_nul();

    let string_target: &mut [u8] =
        unsafe { slice::from_raw_parts_mut(address, (max_to_write).min(cstr.len())) };

    string_target.copy_from_slice(&cstr[..string_target.len()]);
}
