use parking_lot::RwLock;
use std::path::PathBuf;

/* WHY: The default implementation of the CacheFacade using a per-key file store for persistence. */
pub struct DefaultCacheService {
    pub(super) memory: RwLock<Vec<(String, String)>>,
    pub(super) persistent_base_path: PathBuf,
    pub(super) persistent: RwLock<Vec<(String, String)>>,
}

pub struct PlatformCachePathResolver;

impl PlatformCachePathResolver {
    fn app_name() -> String {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "KatanA".to_string())
    }
    
    pub fn cache_root() -> PathBuf {
        let app_name = Self::app_name();
        let dir_name = match app_name.as_str() {
            "KatanB" => "KatanB",
            _ => "KatanA",
        };
        dirs::cache_dir()
            .unwrap_or(PathBuf::from("."))
            .join(dir_name)
    }

    pub fn cache_json_path() -> PathBuf {
        Self::cache_root().join("cache.json")
    }
}
