//! `rupa_metadata` contract: canonical language tables and frame layout.

use rupa_wasm::abi::{rupa_free_frame, rupa_metadata};
use rupa_wasm::{FRAME_HEADER_BYTES, encode_metadata_frame, metadata_frame, metadata_json};

#[test]
fn metadata_matches_the_compiler_tables() {
    let metadata: serde_json::Value =
        serde_json::from_str(&metadata_json()).expect("metadata is JSON");

    assert!(
        metadata["keyword_classes"]["keyword"]
            .as_array()
            .is_some_and(|words| words.iter().any(|word| word == "rectangle"))
    );
    assert!(
        metadata["keyword_classes"]["property"]
            .as_array()
            .is_some_and(|words| words.iter().any(|word| word == "fill"))
    );
    assert_eq!(
        metadata["keyword_classes"]["literal"],
        serde_json::json!(["false", "true"])
    );
    assert!(
        metadata["named_colors"]
            .as_array()
            .is_some_and(|colors| colors.iter().any(|color| color == "green"))
    );
    assert_eq!(
        metadata["units"],
        serde_json::json!(["px", "pt", "%", "deg"])
    );
    assert_eq!(
        metadata["primitives"]["circle"]["properties"][0],
        "center at"
    );
    assert!(
        !metadata["keywords"]
            .as_array()
            .expect("flat keywords")
            .iter()
            .any(|keyword| keyword == "document")
    );
}

#[test]
fn metadata_frame_is_a_header_plus_json_body() {
    let frame = metadata_frame();
    let bytes = encode_metadata_frame(&frame);
    let total = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
    let meta_len = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;

    assert_eq!(total, bytes.len());
    assert_eq!(
        &bytes[FRAME_HEADER_BYTES..FRAME_HEADER_BYTES + meta_len],
        frame.meta.as_bytes()
    );
    assert_eq!(
        &bytes[FRAME_HEADER_BYTES + meta_len..],
        frame.json.as_bytes()
    );
}

#[test]
fn abi_returns_the_same_metadata_body() {
    let expected = metadata_json();
    let pointer = rupa_metadata();
    assert!(!pointer.is_null());

    let total = u32::from_le_bytes(unsafe { *(pointer as *const [u8; 4]) }) as usize;
    let meta_len =
        u32::from_le_bytes(unsafe { *((pointer as *const u8).add(4) as *const [u8; 4]) }) as usize;
    let bytes = unsafe { std::slice::from_raw_parts(pointer, total) };
    let body = std::str::from_utf8(&bytes[FRAME_HEADER_BYTES + meta_len..]).expect("UTF-8 body");
    assert_eq!(body, expected);

    unsafe { rupa_free_frame(pointer) };
}
