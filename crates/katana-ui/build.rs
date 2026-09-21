/* WHY: include! the build-script-only headless process helper so this build.rs can spawn
 * processes (rustc, git) without flashing a console window on Windows. The helper duplicates
 * the minimal CREATE_NO_WINDOW logic from `katana_core::system::ProcessService::create_command`
 * because build scripts cannot depend on `katana-core` (would form a build-time cycle). */
include!("build_support/process.rs");

fn main() {
    println!("cargo::rustc-check-cfg=cfg(coverage)");

    if let Ok(output) = create_build_command("rustc").arg("--version").output() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("cargo:rustc-env=KATANA_RUSTC_VERSION={version}");
    }

    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "dev".to_string());
    if let Ok(output) = create_build_command("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        if output.status.success() {
            let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=KATANA_BUILD={}-{}", profile, hash);
        } else {
            println!("cargo:rustc-env=KATANA_BUILD={}", profile);
        }
    } else {
        println!("cargo:rustc-env=KATANA_BUILD={}", profile);
    }

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "macos" {
        println!("cargo:rerun-if-changed=src/macos_menu.m");

        cc::Build::new()
            .file("src/macos_menu.m")
            .flag("-fobjc-arc")
            .compile("macos_menu");

        println!("cargo:rustc-link-lib=framework=Cocoa");
        // NOTE: 不再嵌入 Info.plist，让系统从 .app bundle 中读取
        // 这样 KatanA 和 KatanB 可以有不同的图标和配置
    }

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        println!("cargo:rerun-if-changed=../../assets/icon.ico");
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../assets/icon.ico");
        let _ = res.compile();
    }
}
