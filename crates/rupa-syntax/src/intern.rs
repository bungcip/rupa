//! Interned strings for identifiers and finite names.
//!
//! `IStr` is a compact handle to a globally interned UTF-8 string. Equal values
//! share one allocation, which keeps repeated identifiers cheap in the token
//! stream and gives the AST a single string representation.
//!
//! Interning is deliberately limited to identifiers and finite names. Large
//! unique payloads such as text content or resource paths stay as `Box<str>` so
//! the global table does not retain them forever (DECISION-042).
//!
//! The interner is intentionally lean (DECISION-044): `ustr`'s fixed global
//! cache costs a multi-megabyte table and several milliseconds on the first
//! intern, which dominated one-shot CLI runs. This implementation starts empty,
//! grows only to the number of unique identifiers in the source, and keeps the
//! same 8-byte `Copy` handle, process-lifetime retention, and string semantics.

use std::collections::HashMap;
use std::fmt;
use std::hash::{BuildHasherDefault, DefaultHasher, Hash, Hasher};
use std::ops::Deref;
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The interned payload. Entries are leaked so handles stay valid for the
/// process lifetime, matching the previous backend's retention policy.
struct Entry {
    hash: u64,
    text: Box<str>,
}

/// Maps interned text to its canonical entry. The fixed hasher avoids the
/// random seed initialization and keeps interning deterministic.
type InternMap = HashMap<&'static str, &'static Entry, BuildHasherDefault<DefaultHasher>>;

static INTERNER: OnceLock<Mutex<InternMap>> = OnceLock::new();

fn interner() -> &'static Mutex<InternMap> {
    INTERNER.get_or_init(|| Mutex::new(InternMap::default()))
}

/// A compact, globally interned string handle.
///
/// `IStr` is 8 bytes on 64-bit targets and 4 bytes on `wasm32`. It serializes as
/// a plain JSON string.
#[derive(Clone, Copy)]
pub struct IStr(&'static Entry);

impl PartialEq for IStr {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl Eq for IStr {}

const _: () = assert!(std::mem::size_of::<IStr>() <= 8);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(std::mem::size_of::<IStr>() <= 4);

impl IStr {
    /// Intern `value` and return its handle.
    ///
    /// Repeated calls with equal strings return handles to the same interned
    /// allocation.
    pub fn new(value: &str) -> Self {
        Self::intern(value)
    }

    /// Return the interned string contents.
    pub fn as_str(&self) -> &str {
        &self.entry().text
    }

    fn intern(value: &str) -> Self {
        let mut entries = interner()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(entry) = entries.get(value) {
            return Self::from_entry(entry);
        }

        let entry: &'static Entry = Box::leak(Box::new(Entry {
            hash: hash_str(value),
            text: value.into(),
        }));
        entries.insert(&entry.text, entry);
        Self::from_entry(entry)
    }

    fn from_entry(entry: &'static Entry) -> Self {
        Self(entry)
    }

    fn entry(&self) -> &'static Entry {
        self.0
    }
}

impl From<&str> for IStr {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for IStr {
    fn from(value: String) -> Self {
        Self::new(&value)
    }
}

impl Deref for IStr {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for IStr {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Hash for IStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Use the canonical entry's precomputed hash so scopes and function
        // tables do not re-hash identifier text on every lookup.
        state.write_u64(self.entry().hash);
    }
}

fn hash_str(value: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

impl PartialEq<str> for IStr {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for IStr {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl fmt::Display for IStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for IStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl Serialize for IStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for IStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|value| Self::new(&value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn equal_values_share_one_interned_allocation() {
        let first = IStr::new("repeated-identifier");
        let second = IStr::new("repeated-identifier");
        let other = IStr::new("different-identifier");

        assert_eq!(first, second);
        assert_eq!(first.as_str().as_ptr(), second.as_str().as_ptr());
        assert_ne!(first, other);
    }

    #[test]
    fn conversions_and_formatting_match_a_plain_string() {
        let from_str = IStr::from("Card");
        let from_string = IStr::from("Card".to_string());

        assert_eq!(from_str, from_string);
        assert_eq!(&*from_str, "Card");
        assert_eq!(from_str.as_ref(), "Card");
        assert_eq!(from_str.to_string(), "Card");
        assert_eq!(format!("{from_str:?}"), "\"Card\"");
    }

    #[test]
    fn serializes_as_a_plain_json_string_and_round_trips() {
        let value = IStr::new("rounded");
        let json = serde_json::to_string(&value).expect("serialize");
        assert_eq!(json, "\"rounded\"");

        let parsed: IStr = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, value);
        assert_eq!(parsed.as_str().as_ptr(), value.as_str().as_ptr());
    }

    #[test]
    fn interning_is_thread_safe_and_canonical() {
        let handles: Vec<usize> = std::thread::scope(|scope| {
            let workers: Vec<_> = (0..8)
                .map(|_| {
                    scope.spawn(|| {
                        let mut pointer = 0;
                        for _ in 0..1_000 {
                            pointer = IStr::new("shared-across-threads").as_str().as_ptr() as usize;
                        }
                        pointer
                    })
                })
                .collect();
            workers
                .into_iter()
                .map(|worker| worker.join().expect("worker"))
                .collect()
        });

        assert!(
            handles.iter().all(|pointer| *pointer == handles[0]),
            "concurrent interning must return one canonical allocation"
        );
    }

    #[test]
    fn handle_is_compact() {
        assert!(size_of::<IStr>() <= 8);
    }
}
