//! C ABI for the JavaScript host.
//!
//! The module is the only place in the workspace that owns raw pointers. Every
//! buffer that crosses the boundary is allocated here with the global allocator
//! and freed here, so the host never guesses a Rust layout.
//!
//! Responsibility of the host after loading the module:
//!
//! 1. `rupa_alloc(len)` → write UTF-8 source bytes into that memory.
//! 2. `rupa_render(ptr, len)` → pointer to a frame (or 0 on allocation
//!    failure). Read `u32 total length` and `u32 metadata length` at the
//!    pointer, then copy the bytes out before making another call (a growing
//!    linear memory invalidates typed-array views).
//! 3. `rupa_format(ptr, len)` → pointer to the same header shape whose body is
//!    the canonical `rupa fmt` text (empty when formatting reported errors).
//! 4. `rupa_metadata()` → pointer to the same header shape whose body is the
//!    canonical language metadata JSON (keyword classes, colours, units, and
//!    primitive properties).
//! 5. `rupa_free_frame(frame)` and `rupa_free(ptr, len)` to release memory.

use crate::{
    encode_format_frame, encode_frame, encode_metadata_frame, format_source_text, metadata_frame,
    render_source,
};
use std::alloc::{Layout, alloc, dealloc};

/// Allocator layout used for every buffer handed to the host.
///
/// A zero-length request still allocates one byte so the returned pointer is
/// non-null and can be freed with the same length.
fn layout(len: usize) -> Option<Layout> {
    Layout::from_size_align(len.max(1), 1).ok()
}

/// Allocate host-owned memory for `len` bytes.
///
/// Returns a null pointer only when the allocator rejects the layout.
#[unsafe(no_mangle)]
pub extern "C" fn rupa_alloc(len: usize) -> *mut u8 {
    match layout(len) {
        Some(layout) => unsafe { alloc(layout) },
        None => std::ptr::null_mut(),
    }
}

/// Free memory previously returned by [`rupa_alloc`].
///
/// # Safety
///
/// `ptr` must be a pointer returned by [`rupa_alloc`] with the same `len`, and
/// it must not have been freed already.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rupa_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() {
        return;
    }
    if let Some(layout) = layout(len) {
        unsafe { dealloc(ptr, layout) };
    }
}

/// Read a host buffer as UTF-8 source text.
///
/// # Safety
///
/// `ptr` must point to `len` readable bytes, or be null when `len` is zero.
unsafe fn read_source(ptr: *const u8, len: usize) -> Option<String> {
    if len == 0 {
        return Some(String::new());
    }
    if ptr.is_null() {
        return None;
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    Some(String::from_utf8_lossy(bytes).into_owned())
}

/// Copy bytes into a fresh host-owned allocation.
///
/// Returns null when the allocator rejects the layout.
fn copy_out(bytes: &[u8]) -> *mut u8 {
    let Some(layout) = layout(bytes.len()) else {
        return std::ptr::null_mut();
    };
    let buffer = unsafe { alloc(layout) };
    if buffer.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer, bytes.len()) };
    buffer
}

/// Compile and render a source string supplied by the host.
///
/// Returns a pointer to a frame buffer, or null when allocation fails. The
/// buffer is released with [`rupa_free_frame`].
///
/// # Safety
///
/// `src_ptr` must point to `src_len` readable bytes that stay valid for the
/// duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rupa_render(src_ptr: *const u8, src_len: usize) -> *mut u8 {
    let Some(source) = (unsafe { read_source(src_ptr, src_len) }) else {
        return std::ptr::null_mut();
    };
    copy_out(&encode_frame(&render_source(&source)))
}

/// Format a source string exactly as `rupa fmt` does.
///
/// Returns a pointer to a frame whose header is the same as [`rupa_render`]'s
/// and whose body is the canonical UTF-8 text (empty when formatting reported
/// errors; the metadata then lists them). Release it with [`rupa_free_frame`].
///
/// # Safety
///
/// `src_ptr` must point to `src_len` readable bytes that stay valid for the
/// duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rupa_format(src_ptr: *const u8, src_len: usize) -> *mut u8 {
    let Some(source) = (unsafe { read_source(src_ptr, src_len) }) else {
        return std::ptr::null_mut();
    };
    copy_out(&encode_format_frame(&format_source_text(&source)))
}

/// Return canonical language metadata to the host.
///
/// The buffer uses the same header as [`rupa_render`]; its body is JSON with
/// keyword classes, named colours, units, and primitive properties. Release it
/// with [`rupa_free_frame`]. Returns null when allocation fails.
#[unsafe(no_mangle)]
pub extern "C" fn rupa_metadata() -> *mut u8 {
    copy_out(&encode_metadata_frame(&metadata_frame()))
}

/// Free a frame returned by [`rupa_render`].
///
/// # Safety
///
/// `frame` must be a pointer returned by [`rupa_render`] and not freed already.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rupa_free_frame(frame: *mut u8) {
    if frame.is_null() {
        return;
    }
    let mut header = [0u8; 4];
    unsafe { std::ptr::copy_nonoverlapping(frame, header.as_mut_ptr(), 4) };
    let total = u32::from_le_bytes(header) as usize;
    if let Some(layout) = layout(total) {
        unsafe { dealloc(frame, layout) };
    }
}

/// Install font bytes used for `text` primitives in the browser.
///
/// `family` may be empty to install the default face. Returns `1` on success
/// and `0` when the input is malformed.
///
/// # Safety
///
/// `family_ptr` must point to `family_len` readable bytes and `data_ptr` to
/// `data_len` readable bytes, both valid for the duration of the call.
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rupa_set_font(
    family_ptr: *const u8,
    family_len: usize,
    bold: u32,
    data_ptr: *const u8,
    data_len: usize,
) -> u32 {
    if data_len == 0 || data_ptr.is_null() {
        return 0;
    }
    if family_len > 0 && family_ptr.is_null() {
        return 0;
    }
    let family = if family_len == 0 {
        String::new()
    } else {
        let bytes = unsafe { std::slice::from_raw_parts(family_ptr, family_len) };
        String::from_utf8_lossy(bytes).into_owned()
    };
    let data = unsafe { std::slice::from_raw_parts(data_ptr, data_len) }.to_vec();
    rupa_renderer::install_font(&family, bold != 0, data);
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FRAME_HEADER_BYTES, render_source};

    #[test]
    fn frame_header_is_a_total_length_and_meta_length() {
        let frame = render_source("size 20px x 10px\nbackground red\n");
        let bytes = encode_frame(&frame);
        let total = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
        let meta_len = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        assert_eq!(total, bytes.len());
        assert_eq!(meta_len, frame.meta.len());
        assert_eq!(
            &bytes[FRAME_HEADER_BYTES..FRAME_HEADER_BYTES + meta_len],
            frame.meta.as_bytes()
        );
        assert_eq!(
            &bytes[FRAME_HEADER_BYTES + meta_len..],
            frame.png.as_slice()
        );
    }

    #[test]
    fn alloc_and_free_roundtrip() {
        let pointer = rupa_alloc(16);
        assert!(!pointer.is_null());
        unsafe { rupa_free(pointer, 16) };
    }

    #[test]
    fn render_through_the_abi_matches_the_safe_api() {
        let source = "size 12px x 12px\nbackground blue\n";
        let expected = encode_frame(&render_source(source));

        let source_pointer = rupa_alloc(source.len());
        assert!(!source_pointer.is_null());
        unsafe {
            std::ptr::copy_nonoverlapping(source.as_ptr(), source_pointer, source.len());
        }
        let frame = unsafe { rupa_render(source_pointer, source.len()) };
        assert!(!frame.is_null());
        let total = u32::from_le_bytes(unsafe { *(frame as *const [u8; 4]) }) as usize;
        let actual = unsafe { std::slice::from_raw_parts(frame, total) }.to_vec();
        assert_eq!(actual, expected);

        unsafe {
            rupa_free_frame(frame);
            rupa_free(source_pointer, source.len());
        }
    }
}
