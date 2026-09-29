/* WHY: Diagram render caches (persistent `diagrams/` root and the temporary Mermaid/Draw.io
 * image directories) live in one place so the key filter, the per-app temp isolation and the
 * cleanup stay consistent with each other. */

use super::types::DefaultCacheService;
use std::path::PathBuf;

impl DefaultCacheService {
    pub(super) fn is_diagram_cache_raw_key(key: &str) -> bool {
        key.starts_with("diagram:") || key.starts_with("diagram_render:")
    }

    /// Temp directories the diagram renderers write images into, isolated per app instance.
    pub(super) fn temporary_diagram_cache_dirs() -> [PathBuf; 2] {
        let temp_dir = std::env::temp_dir();
        let app_suffix = Self::temporary_cache_app_suffix();
        [
            temp_dir.join(format!("{app_suffix}_mermaid_cache")),
            temp_dir.join(format!("{app_suffix}_drawio_cache")),
        ]
    }

    /// Detect app name for temp directory isolation
    pub(super) fn temporary_cache_app_suffix() -> String {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .map(|name| match name.as_str() {
                "KatanB" => "katana_b".to_string(),
                _ => "katana_a".to_string(),
            })
            .unwrap_or_else(|| "katana".to_string())
    }

    pub(super) fn clear_temporary_diagram_images() {
        for cache_dir in Self::temporary_diagram_cache_dirs() {
            let _ = std::fs::remove_dir_all(cache_dir);
        }
    }
}
