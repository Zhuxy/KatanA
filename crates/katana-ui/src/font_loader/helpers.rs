use std::fs;

use egui::{FontData, FontDefinitions, FontFamily};

use super::types::SystemFontLoader;

impl SystemFontLoader {
    pub(super) fn load_first_valid(
        fonts: &mut FontDefinitions,
        candidates: &[&str],
        tweak: Option<egui::FontTweak>,
        suffix: &str,
    ) -> Option<String> {
        for &path in candidates {
            let Ok(data) = fs::read(path) else { continue };
            let name = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("cjk_font")
                .to_string()
                + suffix;
            let mut font_data = FontData::from_owned(data);
            if let Some(t) = tweak {
                font_data.tweak = t;
            }
            fonts
                .font_data
                .insert(name.clone(), std::sync::Arc::new(font_data));
            return Some(name);
        }
        None
    }

    /// Load all valid fonts from candidates as fallbacks.
    ///
    /// WHY: Fallback faces must reuse the family's primary `FontTweak` (especially the
    /// baseline `y_offset_factor`). Otherwise a glyph that falls back to another face
    /// (e.g. a Simplified-Chinese-only character next to Japanese text) is drawn at a
    /// different height, which looks like jitter inside a single line.
    pub(super) fn load_all_valid_as_fallbacks(
        fonts: &mut FontDefinitions,
        candidates: &[&str],
        family: FontFamily,
        tweak: Option<egui::FontTweak>,
    ) {
        for &path in candidates {
            let Ok(data) = fs::read(path) else { continue };
            let name = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("cjk_font")
                .to_string();

            // Skip if already loaded
            if fonts.font_data.contains_key(&name) {
                continue;
            }

            let mut font_data = FontData::from_owned(data);
            if let Some(t) = tweak.clone() {
                font_data.tweak = t;
            }
            fonts
                .font_data
                .insert(name.clone(), std::sync::Arc::new(font_data));
            Self::append_fallback(fonts, family.clone(), &name);
        }
    }

    pub(super) fn prepend_primary(fonts: &mut FontDefinitions, family: FontFamily, name: &str) {
        if let Some(list) = fonts.families.get_mut(&family) {
            list.insert(0, name.to_string());
        }
    }

    pub(super) fn append_fallback(fonts: &mut FontDefinitions, family: FontFamily, name: &str) {
        if let Some(list) = fonts.families.get_mut(&family) {
            list.push(name.to_string());
        }
    }

    /// Load every valid candidate as a distinct fallback face with `tweak` and `suffix`.
    ///
    /// WHY: No single face covers every CJK script. Registering every candidate lets a
    /// glyph missing from the primary face (e.g. a Simplified-Chinese-only character)
    /// resolve to a real glyph instead of the `.notdef` tofu box. Faces are inserted right
    /// after the primary so CJK faces win over unrelated Latin/emoji fallbacks.
    pub(super) fn load_all_valid_as_tweaked_fallbacks(
        fonts: &mut FontDefinitions,
        candidates: &[&str],
        family: FontFamily,
        tweak: egui::FontTweak,
        suffix: &str,
    ) {
        let mut loaded = Vec::new();
        for &path in candidates {
            let Ok(data) = fs::read(path) else { continue };
            let name = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("cjk_font")
                .to_string()
                + suffix;
            if fonts.font_data.contains_key(&name) {
                continue;
            }
            let mut font_data = FontData::from_owned(data);
            font_data.tweak = tweak.clone();
            fonts
                .font_data
                .insert(name.clone(), std::sync::Arc::new(font_data));
            loaded.push(name);
        }

        if let Some(list) = fonts.families.get_mut(&family) {
            for (offset, name) in loaded.into_iter().enumerate() {
                let position = (1 + offset).min(list.len());
                list.insert(position, name);
            }
        }
    }

    pub(super) fn inject_custom_font(fonts: &mut FontDefinitions, path: &str, name: &str) {
        let Ok(data) = fs::read(path) else { return };
        let mut font_data = FontData::from_owned(data);
        font_data.tweak.y_offset = super::normalize::LINUX_Y_OFFSET;
        fonts
            .font_data
            .insert(name.to_string(), std::sync::Arc::new(font_data));
        Self::prepend_primary(fonts, FontFamily::Proportional, name);
    }
}
