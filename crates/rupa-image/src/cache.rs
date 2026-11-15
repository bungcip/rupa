//! Shared path resolution and caching for file-backed resources.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Resolves references against a base directory and caches loaded values by
/// resolved path.
#[derive(Debug, Clone)]
pub(crate) struct PathCache<T> {
    base_dir: PathBuf,
    entries: BTreeMap<PathBuf, T>,
}

impl<T> Default for PathCache<T> {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::new(),
            entries: BTreeMap::new(),
        }
    }
}

impl<T> PathCache<T> {
    pub(crate) fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
            entries: BTreeMap::new(),
        }
    }

    pub(crate) fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    pub(crate) fn resolve(&self, reference: &str) -> PathBuf {
        self.base_dir.join(reference)
    }

    /// Load a path unless it is already cached. The loader runs only on a miss.
    pub(crate) fn load_path_with<E>(
        &mut self,
        path: &Path,
        load: impl FnOnce(&Path) -> Result<T, E>,
    ) -> Result<&T, E> {
        if !self.entries.contains_key(path) {
            let value = load(path)?;
            self.entries.insert(path.to_path_buf(), value);
        }
        Ok(&self.entries[path])
    }

    pub(crate) fn get(&self, reference: &str) -> Option<&T> {
        self.entries.get(&self.resolve(reference))
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn paths(&self) -> impl Iterator<Item = &Path> {
        self.entries.keys().map(PathBuf::as_path)
    }
}
