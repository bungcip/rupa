//! `rupa_format` contract: canonical text, idempotency, error payload, and ABI.

use rupa_wasm::abi::{rupa_alloc, rupa_format, rupa_free, rupa_free_frame};
use rupa_wasm::{FRAME_HEADER_BYTES, FormatFrame, encode_format_frame, format_source_text};

fn meta(frame: &FormatFrame) -> serde_json::Value {
    serde_json::from_str(&frame.meta).expect("metadata is JSON")
}

#[test]
fn formatting_expands_inline_properties() {
    let frame = format_source_text(
        "size 200px x 100px\nrectangle at (10px, 10px) size 80px x 40px fill red\n",
    );

    assert_eq!(meta(&frame)["status"], "ok");
    assert!(
        frame
            .text
            .contains("rectangle at (10px, 10px)\n    size 80px x 40px\n    fill red\n"),
        "unexpected text:\n{}",
        frame.text
    );
}

#[test]
fn formatting_is_idempotent() {
    let once = format_source_text(
        "size 320px x 200px\nbackground white\n\n// note\ncircle center at (10px, 10px) radius 5px fill blue\n",
    );
    let twice = format_source_text(&once.text);

    assert_eq!(meta(&once)["status"], "ok");
    assert_eq!(twice.text, once.text);
}

#[test]
fn formatting_keeps_comments() {
    let frame = format_source_text("size 10px x 10px\n// keep me\nbackground red\n");
    assert_eq!(meta(&frame)["status"], "ok");
    assert!(frame.text.contains("// keep me"), "{}", frame.text);
}

#[test]
fn lexical_errors_return_diagnostics_and_no_text() {
    let frame = format_source_text("\tsize 10px x 10px\n");
    let meta = meta(&frame);

    assert_eq!(meta["status"], "error");
    assert!(
        frame.text.is_empty(),
        "failed formatting must not emit text"
    );
    assert!(
        meta["diagnostics"]
            .as_array()
            .is_some_and(|list| !list.is_empty()),
        "expected diagnostics, got {}",
        frame.meta
    );
}

#[test]
fn syntactically_invalid_source_is_still_reformatted() {
    // `rupa fmt` works on tokens, so a missing `x` is laid out and then reported
    // by the render pass; the formatter must not refuse to help the user.
    let frame = format_source_text("size 10px 10px\n");
    assert_eq!(meta(&frame)["status"], "ok");
    assert!(frame.text.contains("size 10px 10px"), "{}", frame.text);
}

#[test]
fn format_frame_layout_is_a_header_plus_text() {
    let frame = format_source_text("size 10px x 10px\nbackground red\n");
    let bytes = encode_format_frame(&frame);
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
        frame.text.as_bytes()
    );
}

#[test]
fn rupa_format_through_the_abi_matches_the_safe_api() {
    let source = "size 200px x 100px\nrectangle at (10px, 10px) size 80px x 40px fill red\n";
    let expected = encode_format_frame(&format_source_text(source));

    let source_pointer = rupa_alloc(source.len());
    assert!(!source_pointer.is_null());
    unsafe {
        std::ptr::copy_nonoverlapping(source.as_ptr(), source_pointer, source.len());
    }

    let frame = unsafe { rupa_format(source_pointer, source.len()) };
    assert!(!frame.is_null(), "rupa_format returned null");
    let total = u32::from_le_bytes(unsafe { *(frame as *const [u8; 4]) }) as usize;
    let actual = unsafe { std::slice::from_raw_parts(frame, total) }.to_vec();
    assert_eq!(actual, expected);

    unsafe {
        rupa_free_frame(frame);
        rupa_free(source_pointer, source.len());
    }
}
