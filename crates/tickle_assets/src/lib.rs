use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use thiserror::Error;

/// Unique identifier for an asset, wrapping a String.
#[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AssetId(pub String);

impl AssetId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("asset not found: {0}")]
    NotFound(PathBuf),
    #[error("failed to read asset file '{path}': {source}")]
    ReadError {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse RON asset '{path}': {source}")]
    ParseError {
        path: PathBuf,
        source: ron::error::SpannedError,
    },
}

/// Generic asset manager that loads RON files and caches them with Arc<T>.
pub struct AssetManager<T> {
    base_path: PathBuf,
    cache: HashMap<AssetId, Arc<T>>,
}

impl<T: DeserializeOwned> AssetManager<T> {
    /// Create a new AssetManager rooted at the given base path.
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
            cache: HashMap::new(),
        }
    }

    /// Load an asset from a RON file relative to the base path.
    /// The file is read, parsed, cached, and an Arc reference is returned.
    pub fn load(&mut self, id: &AssetId, relative_path: &str) -> Result<Arc<T>, AssetError> {
        if let Some(cached) = self.cache.get(id) {
            return Ok(Arc::clone(cached));
        }

        let full_path = self.base_path.join(relative_path);
        if !full_path.exists() {
            return Err(AssetError::NotFound(full_path));
        }

        let contents = std::fs::read_to_string(&full_path).map_err(|e| AssetError::ReadError {
            path: full_path.clone(),
            source: e,
        })?;

        let asset: T = ron::from_str(&contents).map_err(|e| AssetError::ParseError {
            path: full_path,
            source: e,
        })?;

        let arc = Arc::new(asset);
        self.cache.insert(id.clone(), Arc::clone(&arc));
        Ok(arc)
    }

    /// Get a previously loaded asset from the cache.
    pub fn get(&self, id: &AssetId) -> Option<Arc<T>> {
        self.cache.get(id).map(Arc::clone)
    }

    /// Returns the number of cached assets.
    pub fn cached_count(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::fs;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestCharacter {
        id: String,
        name: String,
        health: i32,
        walk_speed: i32,
    }

    #[test]
    fn test_load_and_cache() {
        let dir = std::env::temp_dir().join("tickle_asset_test_load");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let ron_content = r#"(
    id: "test_char",
    name: "Test Fighter",
    health: 10000,
    walk_speed: 500,
)"#;
        fs::write(dir.join("char.ron"), ron_content).unwrap();

        let mut manager = AssetManager::<TestCharacter>::new(&dir);
        let id = AssetId::new("test_char");

        let asset = manager.load(&id, "char.ron").unwrap();
        assert_eq!(asset.id, "test_char");
        assert_eq!(asset.health, 10000);

        // Second load should return cached version (same Arc)
        let asset2 = manager.load(&id, "char.ron").unwrap();
        assert!(Arc::ptr_eq(&asset, &asset2));
        assert_eq!(manager.cached_count(), 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_get_returns_none_for_unloaded() {
        let manager = AssetManager::<TestCharacter>::new("/nonexistent");
        let id = AssetId::new("missing");
        assert!(manager.get(&id).is_none());
    }

    #[test]
    fn test_get_returns_cached_asset() {
        let dir = std::env::temp_dir().join("tickle_asset_test_get");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let ron_content = r#"(
    id: "cached",
    name: "Cached Fighter",
    health: 8000,
    walk_speed: 400,
)"#;
        fs::write(dir.join("cached.ron"), ron_content).unwrap();

        let mut manager = AssetManager::<TestCharacter>::new(&dir);
        let id = AssetId::new("cached");
        manager.load(&id, "cached.ron").unwrap();

        let got = manager.get(&id).unwrap();
        assert_eq!(got.name, "Cached Fighter");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_not_found_error() {
        let mut manager = AssetManager::<TestCharacter>::new("/nonexistent");
        let id = AssetId::new("nope");
        let err = manager.load(&id, "nope.ron").unwrap_err();
        assert!(matches!(err, AssetError::NotFound(_)));
    }

    #[test]
    fn test_parse_error() {
        let dir = std::env::temp_dir().join("tickle_asset_test_parse");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("bad.ron"), "this is not valid RON {{{{").unwrap();

        let mut manager = AssetManager::<TestCharacter>::new(&dir);
        let id = AssetId::new("bad");
        let err = manager.load(&id, "bad.ron").unwrap_err();
        assert!(matches!(err, AssetError::ParseError { .. }));

        let _ = fs::remove_dir_all(&dir);
    }
}
