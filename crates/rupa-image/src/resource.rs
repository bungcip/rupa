//! Resource loading for Rupa.
//!
//! A [`ResourceStore`] resolves resource paths relative to a project directory
//! and caches decoded images, so instantiating one picture many times decodes
//! the file once. Resolution and decoding are deterministic: the same base
//! directory and reference always produce the same buffer (specification §2).

use crate::cache::PathCache;
use crate::decode::{ImageBuffer, ImageError, decode_image};
use std::path::{Path, PathBuf};

/// Resolves and caches image resources for one document.
#[derive(Debug, Clone, Default)]
pub struct ResourceStore {
    cache: PathCache<ImageBuffer>,
}

impl ResourceStore {
    /// Creates a store whose relative references resolve against `base_dir`.
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache: PathCache::new(base_dir),
        }
    }

    /// Directory used to resolve relative resource references.
    pub fn base_dir(&self) -> &Path {
        self.cache.base_dir()
    }

    /// Resolves a resource reference to a filesystem path.
    ///
    /// Absolute references are kept as-is; relative references are joined to
    /// [`Self::base_dir`]. Paths are not canonicalized, so diagnostics show the
    /// path as written.
    pub fn resolve(&self, reference: &str) -> PathBuf {
        self.cache.resolve(reference)
    }

    /// Loads a reference relative to the store's base directory.
    ///
    /// The decoded image is cached; repeated loads of the same resolved path do
    /// not touch the filesystem again.
    pub fn load(&mut self, reference: &str) -> Result<&ImageBuffer, ImageError> {
        let path = self.resolve(reference);
        self.load_path(&path)
    }

    /// Loads an image from an explicit path, caching it by that path.
    pub fn load_path(&mut self, path: &Path) -> Result<&ImageBuffer, ImageError> {
        self.cache.load_path_with(path, decode_image)
    }

    /// Returns a previously loaded image for `reference`, if any.
    pub fn get(&self, reference: &str) -> Option<&ImageBuffer> {
        self.cache.get(reference)
    }

    /// Number of decoded images currently cached.
    pub fn loaded_count(&self) -> usize {
        self.cache.len()
    }

    /// Resolved paths of all cached images, sorted.
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
            std::env::temp_dir().join(format!("rupa-image-resources-{}-{id}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn write_red_png(path: &Path) {
        let pixels = [255u8, 0, 0, 255];
        let file = std::fs::File::create(path).expect("create PNG");
        let writer = std::io::BufWriter::new(file);
        crate::encode::encode_rgba_png_to_writer(&pixels, 1, 1, writer).expect("encode PNG");
    }

    #[test]
    fn resolves_references_against_base_dir() {
        let store = ResourceStore::new("assets");
        assert_eq!(
            store.resolve("photo.png"),
            PathBuf::from("assets/photo.png")
        );
        assert_eq!(
            store.resolve("/tmp/photo.png"),
            PathBuf::from("/tmp/photo.png")
        );
        assert_eq!(store.base_dir(), Path::new("assets"));
    }

    #[test]
    fn loads_and_caches_images_by_resolved_path() {
        let dir = temp_dir();
        write_red_png(&dir.join("red.png"));
        let mut store = ResourceStore::new(&dir);

        let first = store.load("red.png").expect("load").clone();
        assert_eq!(first.pixel(0, 0), Some([255, 0, 0, 255]));
        assert_eq!(first.width, 1);

        std::fs::remove_file(dir.join("red.png")).expect("remove source");
        // Cached: loading again succeeds without the file.
        let second = store.load("red.png").expect("cached load");
        assert_eq!(second, &first);
        assert_eq!(store.loaded_count(), 1);
        assert_eq!(
            store.loaded_paths().collect::<Vec<_>>(),
            vec![dir.join("red.png").as_path()]
        );
        assert_eq!(store.get("red.png"), Some(&first));
        assert_eq!(store.get("other.png"), None);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn missing_resource_error_reports_resolved_path() {
        let dir = temp_dir();
        let mut store = ResourceStore::new(&dir);

        let error = store.load("missing.png").expect_err("must fail");

        assert_eq!(error.path(), dir.join("missing.png").as_path());
        assert!(matches!(error, ImageError::Read { .. }), "{error}");
        assert_eq!(store.loaded_count(), 0);
        let _ = std::fs::remove_dir_all(dir);
    }
}
