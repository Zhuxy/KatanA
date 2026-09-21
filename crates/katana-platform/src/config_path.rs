use std::ffi::OsString;
use std::path::PathBuf;

const CONFIG_DIR_ENV: &str = "KATANA_CONFIG_DIR";

pub(crate) struct AppConfigPath;

impl AppConfigPath {
    pub(crate) fn resolve() -> PathBuf {
        resolve_app_config_dir(std::env::var_os(CONFIG_DIR_ENV), dirs::config_dir())
    }
}

fn resolve_app_config_dir(
    configured: Option<OsString>,
    platform_config_dir: Option<PathBuf>,
) -> PathBuf {
    configured
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let base = platform_config_dir
                .unwrap_or_else(|| PathBuf::from("."));
            
            // Detect app name from executable path to support multiple instances
            let app_name = std::env::current_exe()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                .unwrap_or_else(|| "KatanA".to_string());
            
            // Use app name as config directory suffix
            match app_name.as_str() {
                "KatanB" => base.join("KatanB"),
                _ => base.join("KatanA"),
            }
        })
}

#[cfg(test)]
mod tests {
    use super::resolve_app_config_dir;
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn explicit_config_directory_wins_over_platform_default() {
        let resolved = resolve_app_config_dir(
            Some(OsString::from("/tmp/katana-evidence")),
            Some(PathBuf::from("/Users/test/Library/Application Support")),
        );

        assert_eq!(resolved, PathBuf::from("/tmp/katana-evidence"));
    }

    #[test]
    fn platform_default_keeps_katana_subdirectory() {
        let resolved = resolve_app_config_dir(
            None,
            Some(PathBuf::from("/Users/test/Library/Application Support")),
        );

        assert_eq!(
            resolved,
            PathBuf::from("/Users/test/Library/Application Support/KatanA")
        );
    }
}
