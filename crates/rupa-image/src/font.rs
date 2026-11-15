//! Font loading for Rupa (specification §9, §10).
//!
//! A [`FontStore`] resolves font references against a project directory and
//! caches raw font bytes, so the same font is read once per document.

use crate::cache::PathCache;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Errors that can occur while loading a font.
#[derive(Debug)]
pub enum FontError {
    /// The font file could not be read.
    Read {
        /// Resolved filesystem path.
        path: PathBuf,
        /// Underlying I/O error.
        source: io::Error,
    },
}

impl FontError {
    /// Resolved path associated with the failure.
    pub fn path(&self) -> &Path {
        match self {
            FontError::Read { path, .. } => path,
        }
    }
}

impl std::fmt::Display for FontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FontError::Read { path, source } => {
                write!(f, "failed to read font {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for FontError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FontError::Read { source, .. } => Some(source),
        }
    }
}

/// Resolves and caches font resources for one document.
#[derive(Debug, Clone, Default)]
pub struct FontStore {
    cache: PathCache<Vec<u8>>,
}

impl FontStore {
    /// Creates a store whose relative references resolve against `base_dir`.
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache: PathCache::new(base_dir),
        }
    }

    /// Directory used to resolve relative font references.
    pub fn base_dir(&self) -> &Path {
        self.cache.base_dir()
    }

    /// Resolves a font reference to a filesystem path.
    pub fn resolve(&self, reference: &str) -> PathBuf {
        self.cache.resolve(reference)
    }

    /// Loads a font relative to the store's base directory.
    pub fn load(&mut self, reference: &str) -> Result<&[u8], FontError> {
        let path = self.resolve(reference);
        self.load_path(&path)
    }

    /// Loads a font from an explicit path, caching it by that path.
    pub fn load_path(&mut self, path: &Path) -> Result<&[u8], FontError> {
        self.cache
            .load_path_with(path, |path| {
                fs::read(path).map_err(|source| FontError::Read {
                    path: path.to_path_buf(),
                    source,
                })
            })
            .map(Vec::as_slice)
    }

    /// Returns a previously loaded font for `reference`, if any.
    pub fn get(&self, reference: &str) -> Option<&[u8]> {
        self.cache.get(reference).map(Vec::as_slice)
    }

    /// Number of fonts currently cached.
    pub fn loaded_count(&self) -> usize {
        self.cache.len()
    }

    /// Resolved paths of all cached fonts, sorted.
    pub fn loaded_paths(&self) -> impl Iterator<Item = &Path> {
        self.cache.paths()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let id = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("rupa-image-fonts-{}-{id}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn missing_font_reports_the_resolved_path() {
        let dir = temp_dir();
        let mut store = FontStore::new(&dir);

        let error = store.load("missing.ttf").expect_err("missing font");
        assert_eq!(error.path(), dir.join("missing.ttf"));
    }
}
