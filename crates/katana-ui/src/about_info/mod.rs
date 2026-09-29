pub const APP_DISPLAY_NAME: &str = "KatanA";

impl AppIdentityOps {
    pub fn app_name() -> &'static str {
        static APP_NAME: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        let name = APP_NAME.get_or_init(|| {
            if let Ok(val) = std::env::var("KATANA_APP_NAME")
                && !val.is_empty()
            {
                return val;
            }
            std::env::current_exe()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                .map(|n| {
                    let stem = n.strip_suffix(".exe").unwrap_or(&n);
                    match stem {
                        "KatanB" => "KatanB".to_string(),
                        _ => APP_DISPLAY_NAME.to_string(),
                    }
                })
                .unwrap_or_else(|| APP_DISPLAY_NAME.to_string())
        });
        name.as_str()
    }
}

/// Identity of the running app instance, so KatanA and KatanB can differ in name and product.
pub struct AppIdentityOps;

pub const APP_PRODUCT_NAME: &str = "KatanA Desktop";

impl AppIdentityOps {
    pub fn product_name() -> &'static str {
        match Self::app_name() {
            "KatanB" => "KatanB Desktop",
            _ => APP_PRODUCT_NAME,
        }
    }
}

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const APP_BUILD: &str = match option_env!("KATANA_BUILD") {
    Some(v) => v,
    None => "dev",
};

pub const APP_COPYRIGHT: &str = "© 2026 KatanA Project";

pub const APP_LICENSE: &str = "MIT License";

pub const APP_REPOSITORY: &str = "https://github.com/HiroyukiFuruno/KatanA";

pub const APP_WEBSITE_URL: &str = "https://katana-desktop.katana-projects.org/";

pub const APP_DOCS_URL: &str = "https://github.com/HiroyukiFuruno/KatanA/tree/master/docs";

pub const APP_ISSUES_URL: &str = "https://github.com/HiroyukiFuruno/KatanA/issues";

pub const APP_SPONSOR_URL: &str = "https://github.com/sponsors/HiroyukiFuruno";

pub const APP_DESCRIPTION: &str = "A fast, keyboard-driven Markdown editor built with Rust.";

mod types;
pub use types::{AboutInfo, AboutInfoOps, SystemInfo};

impl AboutInfoOps {
    pub fn system_info() -> SystemInfo {
        SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            rustc_version: env!("KATANA_RUSTC_VERSION").to_string(),
        }
    }

    pub fn about_info() -> AboutInfo {
        AboutInfo {
            product_name: AppIdentityOps::product_name(),
            version: APP_VERSION,
            build: APP_BUILD,
            copyright: APP_COPYRIGHT,
            license: APP_LICENSE,
            description: APP_DESCRIPTION,
            repository: APP_REPOSITORY,
            website_url: APP_WEBSITE_URL,
            docs_url: APP_DOCS_URL,
            issues_url: APP_ISSUES_URL,
            sponsor_url: APP_SPONSOR_URL,
            system: Self::system_info(),
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
