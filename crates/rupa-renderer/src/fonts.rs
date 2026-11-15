//! Font resolution for text rendering.
//!
//! The default `text` font is the vendored **Rupa Sans** face, embedded in this
//! crate with `include_bytes!`. That makes `text` output independent of the
//! host's installed fonts. The family name and the SHA-256 of each face are
//! pinned here so a build cannot silently ship different bytes.
//!
//! Native builds resolve only these embedded faces, so text output does not
//! depend on system-installed fonts. Browser (`wasm32`) builds also accept
//! faces the host installs through [`install_font`]; those are explicit host
//! resources, not silent system fallback.

use crate::vector::RenderError;

/// Family name of the vendored default font.
pub const DEFAULT_FONT_FAMILY: &str = "Rupa Sans";

/// Vendored regular face used when `text` omits `font` or requests the default.
pub const DEFAULT_FONT_REGULAR: &[u8] = include_bytes!("../assets/RupaSans-Regular.ttf");

/// Vendored bold face used for `weight bold`.
pub const DEFAULT_FONT_BOLD: &[u8] = include_bytes!("../assets/RupaSans-Bold.ttf");

/// Pinned SHA-256 of [`DEFAULT_FONT_REGULAR`].
pub const DEFAULT_FONT_REGULAR_SHA256: &str =
    "f722be64217af6d519e8b4ddd360554058cb0745102dc4cea7f21b08f87caafe";

/// Pinned SHA-256 of [`DEFAULT_FONT_BOLD`].
pub const DEFAULT_FONT_BOLD_SHA256: &str =
    "8c3dec35d02e2af2ce5c0f820126ba669266a807352b754f3280159bddccbd4f";

/// One embedded font face and the digest that pins its bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbeddedFont {
    /// Family name this face is registered under.
    pub family: &'static str,
    /// Whether this is the bold face.
    pub bold: bool,
    /// SHA-256 of [`Self::bytes`], lowercase hexadecimal.
    pub sha256: &'static str,
    /// Font file contents (TTF).
    pub bytes: &'static [u8],
}

/// Embedded faces available without host font discovery or uploads.
pub const EMBEDDED_FONTS: &[EmbeddedFont] = &[
    EmbeddedFont {
        family: DEFAULT_FONT_FAMILY,
        bold: false,
        sha256: DEFAULT_FONT_REGULAR_SHA256,
        bytes: DEFAULT_FONT_REGULAR,
    },
    EmbeddedFont {
        family: DEFAULT_FONT_FAMILY,
        bold: true,
        sha256: DEFAULT_FONT_BOLD_SHA256,
        bytes: DEFAULT_FONT_BOLD,
    },
];

/// Return the embedded face for a family and weight, if one is pinned.
///
/// An empty family (the default) and the canonical [`DEFAULT_FONT_FAMILY`]
/// both select Rupa Sans. An explicit other family returns `None`, leaving
/// native builds to system discovery and browser builds to installed faces.
pub fn embedded_font(family: Option<&str>, bold: bool) -> Option<&'static [u8]> {
    let name = family.unwrap_or_default();
    if !name.is_empty() && !name.eq_ignore_ascii_case(DEFAULT_FONT_FAMILY) {
        return None;
    }
    EMBEDDED_FONTS
        .iter()
        .find(|face| face.bold == bold)
        .or_else(|| EMBEDDED_FONTS.iter().find(|face| !face.bold))
        .map(|face| face.bytes)
}

/// Resolve raw font bytes for a family name (or the embedded default).
///
/// An empty or absent `family` selects Rupa Sans. `bold` selects the bold face
/// when one is available. The same family always maps to the same embedded
/// bytes; native builds do not consult system-installed fonts.
pub fn resolve_font_bytes(family: Option<&str>, bold: bool) -> Result<Vec<u8>, RenderError> {
    #[cfg(target_arch = "wasm32")]
    if let Some(bytes) = registry::lookup(family, bold) {
        return Ok(bytes);
    }

    embedded_font(family, bold)
        .map(<[u8]>::to_vec)
        .ok_or_else(|| {
            RenderError::MissingFont(if family.unwrap_or_default().is_empty() {
                "default font".to_string()
            } else {
                family.unwrap_or_default().to_string()
            })
        })
}

/// Install font bytes for browser rendering, replacing a matching face.
///
/// An empty `family` installs the default face used when text omits `font`.
/// Repeated installs of the same family and weight replace the previous bytes,
/// so a host can hot-swap a font without restarting the renderer.
#[cfg(target_arch = "wasm32")]
pub fn install_font(family: &str, bold: bool, bytes: Vec<u8>) {
    registry::install(family, bold, bytes);
}

#[cfg(target_arch = "wasm32")]
mod registry {
    use std::sync::{Mutex, OnceLock};

    /// One installed font face.
    pub struct Face {
        /// Family name; empty means the default face.
        pub family: String,
        /// Whether this face is a bold face.
        pub bold: bool,
        /// Font file contents (TTF or OTF).
        pub bytes: Vec<u8>,
    }

    fn faces() -> &'static Mutex<Vec<Face>> {
        static FACES: OnceLock<Mutex<Vec<Face>>> = OnceLock::new();
        FACES.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn lock() -> std::sync::MutexGuard<'static, Vec<Face>> {
        // A poisoned registry only means another install panicked while holding
        // the lock; the data is still a valid list of faces.
        faces().lock().unwrap_or_else(|error| error.into_inner())
    }

    /// Install or replace one face.
    pub fn install(family: &str, bold: bool, bytes: Vec<u8>) {
        let mut faces = lock();
        if let Some(existing) = faces
            .iter_mut()
            .find(|face| face.bold == bold && face.family.eq_ignore_ascii_case(family))
        {
            existing.bytes = bytes;
            return;
        }
        faces.push(Face {
            family: family.to_string(),
            bold,
            bytes,
        });
    }

    /// Find a face for a family and weight.
    ///
    /// An empty family matches only installed default faces. An explicit family
    /// matches only that family, so an unknown request does not silently pick
    /// an unrelated installed face; the caller can then use the embedded
    /// default.
    pub fn lookup(family: Option<&str>, bold: bool) -> Option<Vec<u8>> {
        let faces = lock();
        let name = family.unwrap_or_default();
        let matches_family = |face: &Face| face.family.eq_ignore_ascii_case(name);
        faces
            .iter()
            .find(|face| matches_family(face) && face.bold == bold)
            .or_else(|| faces.iter().find(|face| matches_family(face)))
            .map(|face| face.bytes.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_and_named_family_select_the_embedded_faces() {
        assert_eq!(embedded_font(None, false), Some(DEFAULT_FONT_REGULAR));
        assert_eq!(embedded_font(None, true), Some(DEFAULT_FONT_BOLD));
        assert_eq!(
            embedded_font(Some("rupa sans"), false),
            Some(DEFAULT_FONT_REGULAR)
        );
        assert_eq!(embedded_font(Some("Inter"), false), None);
    }

    #[test]
    fn embedded_faces_parse_and_rasterize() {
        for face in EMBEDDED_FONTS {
            let font = fontdue::Font::from_bytes(face.bytes, fontdue::FontSettings::default())
                .unwrap_or_else(|error| {
                    panic!("{} does not parse as a font: {error}", face.family)
                });
            let (metrics, bitmap) = font.rasterize('A', 32.0);
            assert!(
                metrics.advance_width > 0.0,
                "{} has no advance",
                face.family
            );
            assert!(!bitmap.is_empty(), "{} rendered no pixels", face.family);
        }
    }

    #[test]
    fn embedded_faces_match_their_pinned_hashes() {
        for face in EMBEDDED_FONTS {
            assert_eq!(
                sha256_hex(face.bytes),
                face.sha256,
                "embedded font {} is not the pinned bytes",
                face.family
            );
        }
    }

    #[test]
    fn default_font_resolution_uses_the_vendored_bytes() {
        assert_eq!(
            super::resolve_font_bytes(None, false).expect("regular default"),
            DEFAULT_FONT_REGULAR
        );
        assert_eq!(
            super::resolve_font_bytes(None, true).expect("bold default"),
            DEFAULT_FONT_BOLD
        );
    }

    #[test]
    fn unknown_families_do_not_fall_back_to_a_system_font() {
        assert!(matches!(
            super::resolve_font_bytes(Some("Definitely-Not-A-Font-XYZ"), false),
            Err(RenderError::MissingFont(_))
        ));
    }

    /// Minimal SHA-256 for the pinning test, avoiding a production dependency.
    fn sha256_hex(data: &[u8]) -> String {
        const K: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
            0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
            0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
            0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
            0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
            0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
            0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
            0xc67178f2,
        ];
        let mut h: [u32; 8] = [
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
            0x5be0cd19,
        ];

        let bit_len = (data.len() as u64).saturating_mul(8);
        let mut message = data.to_vec();
        message.push(0x80);
        while message.len() % 64 != 56 {
            message.push(0);
        }
        message.extend_from_slice(&bit_len.to_be_bytes());

        for chunk in message.as_chunks::<64>().0 {
            let mut w = [0u32; 64];
            for (index, word) in chunk.as_chunks::<4>().0.iter().enumerate() {
                w[index] = u32::from_be_bytes(*word);
            }
            for index in 16..64 {
                let s0 = w[index - 15].rotate_right(7)
                    ^ w[index - 15].rotate_right(18)
                    ^ (w[index - 15] >> 3);
                let s1 = w[index - 2].rotate_right(17)
                    ^ w[index - 2].rotate_right(19)
                    ^ (w[index - 2] >> 10);
                w[index] = w[index - 16]
                    .wrapping_add(s0)
                    .wrapping_add(w[index - 7])
                    .wrapping_add(s1);
            }

            let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
            for index in 0..64 {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let ch = (e & f) ^ (!e & g);
                let temp1 = hh
                    .wrapping_add(s1)
                    .wrapping_add(ch)
                    .wrapping_add(K[index])
                    .wrapping_add(w[index]);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let temp2 = s0.wrapping_add(maj);
                hh = g;
                g = f;
                f = e;
                e = d.wrapping_add(temp1);
                d = c;
                c = b;
                b = a;
                a = temp1.wrapping_add(temp2);
            }
            h[0] = h[0].wrapping_add(a);
            h[1] = h[1].wrapping_add(b);
            h[2] = h[2].wrapping_add(c);
            h[3] = h[3].wrapping_add(d);
            h[4] = h[4].wrapping_add(e);
            h[5] = h[5].wrapping_add(f);
            h[6] = h[6].wrapping_add(g);
            h[7] = h[7].wrapping_add(hh);
        }

        h.iter().map(|word| format!("{word:08x}")).collect()
    }
}
