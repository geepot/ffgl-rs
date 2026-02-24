use core::slice;
use std::ffi::CString;

/// # Safety
///
/// `address` must be a valid pointer to a buffer of at least `max_to_write` bytes.
pub unsafe fn copy_str_to_host_buffer(address: *mut u8, max_to_write: usize, string: &str) {
    if max_to_write == 0 {
        return;
    }

    let cstr = CString::new(string).unwrap().into_bytes_with_nul();
    let to_copy = cstr.len().min(max_to_write);
    let dest = unsafe { slice::from_raw_parts_mut(address, to_copy) };

    dest.copy_from_slice(&cstr[..to_copy]);

    // If we truncated, ensure the buffer is still null-terminated
    if to_copy < cstr.len() {
        dest[to_copy - 1] = 0;
    }
}
