/* WHY: Verification of common baseline alignment for mixed English and Japanese text. */

use super::*;
#[cfg(target_os = "macos")]
use egui::{Context, FontId};

#[cfg(target_os = "macos")]
fn assert_font_jitter(context_name: &str, font_size: f32) {
    let preset = DiagramColorPreset::current();
    let fonts = SystemFontLoader::build_font_definitions(
        &preset.proportional_font_candidates,
        &preset.monospace_font_candidates,
        &preset.emoji_font_candidates,
        None,
        None,
    );
    let ctx = Context::default();
    ctx.set_fonts(fonts.into_inner());

    let text = format!(
        "Katana — {} Lambda\u{30a2}\u{30c3}\u{30d7}\u{30c7}\u{30fc}\u{30c8}\u{624b}\u{9806}.md",
        context_name
    );
    let mut eng_glyph = None;
    let mut jpn_glyph = None;

    crate::test_ui::TestUiOps::run(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let galley = ui.painter().layout_no_wrap(
                text.clone(),
                FontId::proportional(font_size),
                egui::Color32::WHITE,
            );
            eng_glyph = galley.rows[0].glyphs.iter().find(|g| g.chr == 'L').copied();
            jpn_glyph = galley.rows[0]
                .glyphs
                .iter()
                .find(|g| g.chr == '\u{30a2}')
                .copied();
        });
    });

    let eng_glyph = eng_glyph.expect("English char not found");
    let jpn_glyph = jpn_glyph.expect("Japanese char not found");

    assert_eq!(
        eng_glyph.pos.y, jpn_glyph.pos.y,
        "\u{30ac}\u{30bf}\u{30c4}\u{30ad} (Jitter) in {}: English 'L' y={} vs Japanese '\u{30a2}' y={}",
        context_name, eng_glyph.pos.y, jpn_glyph.pos.y
    );
}

#[test]
#[cfg(target_os = "macos")]
fn test_font_jitter_1_app_title() {
    assert_font_jitter("App Title", 20.0);
}

#[test]
#[cfg(target_os = "macos")]
fn test_font_jitter_2_workspace_dir() {
    assert_font_jitter("Workspace Dir", 14.0);
}

#[test]
#[cfg(target_os = "macos")]
fn test_font_jitter_3_workspace_file() {
    assert_font_jitter("Workspace File", 14.0);
}

#[test]
#[cfg(target_os = "macos")]
fn test_font_jitter_4_toc_heading() {
    assert_font_jitter("TOC Heading", 14.0);
}

#[test]
#[cfg(target_os = "macos")]
fn test_font_jitter_5_tab_name() {
    assert_font_jitter("Tab Name", 14.0);
}

#[test]
#[cfg(target_os = "macos")]
fn test_font_jitter_6_monospace() {
    let preset = DiagramColorPreset::current();
    let fonts = SystemFontLoader::build_font_definitions(
        &preset.proportional_font_candidates,
        &preset.monospace_font_candidates,
        &preset.emoji_font_candidates,
        None,
        None,
    );
    let ctx = Context::default();
    ctx.set_fonts(fonts.into_inner());

    let text =
        "cd infrastructures/tools/crypt-decrypt \u{306e}\u{5fa9}\u{53f7}\u{5316}".to_string();
    let mut eng_glyph = None;
    let mut jpn_glyph = None;

    let mut primitives = vec![];
    crate::test_ui::TestUiOps::run(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let galley = ui.painter().layout_no_wrap(
                text.clone(),
                FontId::monospace(14.0),
                egui::Color32::WHITE,
            );
            eng_glyph = galley.rows[0].glyphs.iter().find(|g| g.chr == 'c').copied();
            jpn_glyph = galley.rows[0]
                .glyphs
                .iter()
                .find(|g| g.chr == '\u{5fa9}')
                .copied();

            let shapes = vec![egui::epaint::ClippedShape {
                clip_rect: egui::Rect::EVERYTHING,
                shape: egui::epaint::Shape::galley(egui::Pos2::ZERO, galley, egui::Color32::WHITE),
            }];
            primitives = ui.ctx().tessellate(shapes, 1.0);
        });
    });

    let eng_glyph = eng_glyph.expect("English char not found");
    let jpn_glyph = jpn_glyph.expect("Japanese char not found");

    let mut eng_min_y = f32::INFINITY;
    let mut jpn_min_y = f32::INFINITY;

    if let egui::epaint::Primitive::Mesh(mesh) = &primitives[0].primitive {
        for v in &mesh.vertices {
            if v.pos.x >= eng_glyph.logical_rect().min.x
                && v.pos.x <= eng_glyph.logical_rect().max.x
            {
                eng_min_y = eng_min_y.min(v.pos.y);
            }
            if v.pos.x >= jpn_glyph.logical_rect().min.x
                && v.pos.x <= jpn_glyph.logical_rect().max.x
            {
                jpn_min_y = jpn_min_y.min(v.pos.y);
            }
        }
    }

    let diff = (eng_min_y - jpn_min_y).abs();
    assert!(
        diff <= 1.5,
        "\u{30ac}\u{30bf}\u{30c4}\u{30ad} (Jitter) in Monospace visual mesh: English 'c' y={} vs Japanese '\u{5fa9}' y={} (Diff: {})",
        eng_min_y,
        jpn_min_y,
        diff
    );
}

#[test]
#[cfg(target_os = "macos")]
fn test_font_jitter_7_codeblock_layoutjob() {
    let preset = DiagramColorPreset::current();
    let fonts = SystemFontLoader::build_font_definitions(
        &preset.proportional_font_candidates,
        &preset.monospace_font_candidates,
        &preset.emoji_font_candidates,
        None,
        None,
    );
    let ctx = Context::default();
    ctx.set_fonts(fonts.into_inner());

    let code_text = "# \u{5168}\u{4ef6}\u{5b9f}\u{884c}";

    let mut hash_glyph = None;
    let mut jp_glyph = None;
    let mut primitives = vec![];

    crate::test_ui::TestUiOps::run(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut job = egui::text::LayoutJob::default();
            job.append(
                code_text,
                0.0,
                egui::TextFormat::simple(
                    egui::TextStyle::Monospace.resolve(ui.style()),
                    egui::Color32::WHITE,
                ),
            );
            job.wrap.max_width = 800.0;

            let galley = ui.fonts_mut(|f| f.layout_job(job));

            if let Some(row) = galley.rows.first() {
                for g in &row.glyphs {
                    if g.chr == '#' {
                        hash_glyph = Some(*g);
                    }
                    if g.chr == '\u{5168}' {
                        jp_glyph = Some(*g);
                    }
                }
            }

            let shapes = vec![egui::epaint::ClippedShape {
                clip_rect: egui::Rect::EVERYTHING,
                shape: egui::epaint::Shape::galley(egui::Pos2::ZERO, galley, egui::Color32::WHITE),
            }];
            primitives = ui.ctx().tessellate(shapes, 1.0);
        });
    });

    let hash_glyph = hash_glyph.expect("'#' glyph not found");
    let jp_glyph = jp_glyph.expect("'\u{5168}' glyph not found");

    let mut hash_min_y = f32::INFINITY;
    let mut jp_min_y = f32::INFINITY;

    if let egui::epaint::Primitive::Mesh(mesh) = &primitives[0].primitive {
        for v in &mesh.vertices {
            if v.pos.x >= hash_glyph.logical_rect().min.x
                && v.pos.x <= hash_glyph.logical_rect().max.x
            {
                hash_min_y = hash_min_y.min(v.pos.y);
            }
            if v.pos.x >= jp_glyph.logical_rect().min.x && v.pos.x <= jp_glyph.logical_rect().max.x
            {
                jp_min_y = jp_min_y.min(v.pos.y);
            }
        }
    }

    let diff = (hash_min_y - jp_min_y).abs();
    assert!(
        diff <= 1.5,
        "\u{30ac}\u{30bf}\u{30c4}\u{30ad} (Jitter) in CodeBlock LayoutJob visual mesh: '#' y={} vs '\u{5168}' y={} (Diff: {}). \
         Mixed JP/EN text must share a common baseline.",
        hash_min_y,
        jp_min_y,
        diff
    );
}

#[test]
#[cfg(target_os = "macos")]
fn test_font_jitter_8_inline_code_cross_family() {
    use egui::TextStyle;
    use egui::text::{LayoutJob, TextFormat};

    let preset = DiagramColorPreset::current();
    let fonts = SystemFontLoader::build_font_definitions(
        &preset.proportional_font_candidates,
        &preset.monospace_font_candidates,
        &preset.emoji_font_candidates,
        None,
        None,
    );
    let ctx = Context::default();
    ctx.set_fonts(fonts.into_inner());

    let mut prop_min_y = f32::INFINITY;
    let mut mono_min_y = f32::INFINITY;

    let mut primitives = vec![];
    crate::test_ui::TestUiOps::run(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut job = LayoutJob::default();

            let prop_format = TextFormat {
                font_id: TextStyle::Body.resolve(ui.style()),
                ..Default::default()
            };
            let mono_format = TextFormat {
                font_id: TextStyle::Monospace.resolve(ui.style()),
                ..Default::default()
            };

            job.append("\u{30a4}\u{30f3}\u{30b9}\u{30c8}\u{30fc}\u{30eb}\u{5f8c}\u{3001}", 0.0, prop_format.clone());
            job.append("mermaid", 0.0, mono_format);
            job.append(" \u{306f}\u{81ea}\u{52d5}\u{7684}\u{306b}\u{691c}\u{51fa}\u{3055}\u{308c}\u{307e}\u{3059}", 0.0, prop_format);

            let galley = ui.fonts_mut(|f| f.layout_job(job));

            let prop_glyph = galley.rows[0]
                .glyphs
                .iter()
                .find(|g| g.chr == '\u{30a4}')
                .copied();
            let mono_glyph = galley.rows[0].glyphs.iter().find(|g| g.chr == 'm').copied();

            if let (Some(pg), Some(mg)) = (prop_glyph, mono_glyph) {
                let shapes = vec![egui::epaint::ClippedShape {
                    clip_rect: egui::Rect::EVERYTHING,
                    shape: egui::epaint::Shape::galley(
                        egui::Pos2::ZERO,
                        galley,
                        egui::Color32::WHITE,
                    ),
                }];
                primitives = ui.ctx().tessellate(shapes, 1.0);

                if let Some(egui::epaint::Primitive::Mesh(mesh)) =
                    primitives.first().map(|p| &p.primitive)
                {
                    for v in &mesh.vertices {
                        if v.pos.x >= pg.logical_rect().min.x
                            && v.pos.x <= pg.logical_rect().max.x
                        {
                            prop_min_y = prop_min_y.min(v.pos.y);
                        }
                        if v.pos.x >= mg.logical_rect().min.x
                            && v.pos.x <= mg.logical_rect().max.x
                        {
                            mono_min_y = mono_min_y.min(v.pos.y);
                        }
                    }
                }
            }
        });
    });

    assert!(
        prop_min_y.is_finite() && mono_min_y.is_finite(),
        "Both glyphs must be found in mesh"
    );

    let diff = (prop_min_y - mono_min_y).abs();
    assert!(
        diff <= 10.0,
        "\u{30ac}\u{30bf}\u{30c4}\u{30ad} (Jitter) in inline code: Proportional '\u{30a4}' y={} vs Monospace 'm' y={} (Diff: {}). \
         Cross-family alignment is not enforced at font-level; handle at LayoutJob level.",
        prop_min_y,
        mono_min_y,
        diff
    );
}

#[test]
#[cfg(target_os = "macos")]
fn test_font_jitter_9_simplified_chinese_fallback_matches_primary_baseline() {
    /* WHY: Simplified-Chinese-only glyphs (e.g. 标/输) are absent from the Japanese
     * primary face and fall back to another CJK font. If that fallback is loaded without
     * the primary face's y_offset tweak, the glyphs visibly jump up/down inside one line. */
    let preset = DiagramColorPreset::current();
    let fonts = SystemFontLoader::build_font_definitions(
        &preset.proportional_font_candidates,
        &preset.monospace_font_candidates,
        &preset.emoji_font_candidates,
        None,
        None,
    );
    let ctx = Context::default();
    ctx.set_fonts(fonts.into_inner());

    /* WHY: 指 exists in the Japanese primary face; 标 only exists in the Chinese fallback. */
    let text = "\u{6307}\u{6807}".to_string();
    let mut primary_glyph = None;
    let mut fallback_glyph = None;

    crate::test_ui::TestUiOps::run(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut job = egui::text::LayoutJob::default();
            job.append(
                &text,
                0.0,
                egui::TextFormat::simple(egui::FontId::proportional(20.0), egui::Color32::WHITE),
            );
            let galley = ui.fonts_mut(|f| f.layout_job(job));
            for glyph in &galley.rows[0].glyphs {
                match glyph.chr {
                    '\u{6307}' => primary_glyph = Some(*glyph),
                    '\u{6807}' => fallback_glyph = Some(*glyph),
                    _ => {}
                }
            }
        });
    });

    let primary_glyph = primary_glyph.expect("primary glyph 指 not found");
    let fallback_glyph = fallback_glyph.expect("fallback glyph 标 not found");

    /* WHY: The pen sits on the baseline, so baseline + uv offset.y is the drawn ink top. */
    let ink_top = |glyph: &egui::epaint::text::Glyph| glyph.pos.y + glyph.uv_rect.offset.y;
    let primary_top = ink_top(&primary_glyph);
    let fallback_top = ink_top(&fallback_glyph);
    let diff = (primary_top - fallback_top).abs();

    assert!(
        diff <= 1.5,
        "\u{30ac}\u{30bf}\u{30c4}\u{30ad} (Jitter): primary '指' ink top={primary_top} vs Chinese fallback '标' ink top={fallback_top} (diff {diff}). \
         CJK fallback faces must reuse the primary face's y_offset tweak so mixed Chinese/Japanese text shares one baseline."
    );
}

#[test]
#[cfg(target_os = "macos")]
fn test_font_jitter_10_monospace_resolves_simplified_chinese_glyphs() {
    /* WHY: Source/code mode renders with the Monospace family. If no Simplified-Chinese
     * capable fallback is registered there, Chinese-only glyphs fall back to the `.notdef`
     * "tofu" box even though the preview shows them through the Proportional family. */
    let preset = DiagramColorPreset::current();
    let fonts = SystemFontLoader::build_font_definitions(
        &preset.proportional_font_candidates,
        &preset.monospace_font_candidates,
        &preset.emoji_font_candidates,
        None,
        None,
    );
    let ctx = Context::default();
    ctx.set_fonts(fonts.into_inner());

    /* WHY: U+E000 is a private-use code point absent from every face, so its glyph is the
     * `.notdef` box. 标 must not reuse that same box. */
    let text = "\u{6807}\u{e000}".to_string();
    let mut chinese_glyph = None;
    let mut notdef_glyph = None;

    crate::test_ui::TestUiOps::run(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut job = egui::text::LayoutJob::default();
            job.append(
                &text,
                0.0,
                egui::TextFormat::simple(egui::FontId::monospace(20.0), egui::Color32::WHITE),
            );
            let galley = ui.fonts_mut(|f| f.layout_job(job));
            for glyph in &galley.rows[0].glyphs {
                match glyph.chr {
                    '\u{6807}' => chinese_glyph = Some(*glyph),
                    '\u{e000}' => notdef_glyph = Some(*glyph),
                    _ => {}
                }
            }
        });
    });

    let chinese = chinese_glyph.expect("Chinese glyph 标 not found");
    let notdef = notdef_glyph.expect("notdef glyph U+E000 not found");
    let chinese_rect = (chinese.uv_rect.offset, chinese.uv_rect.size);
    let notdef_rect = (notdef.uv_rect.offset, notdef.uv_rect.size);

    assert_ne!(
        chinese_rect, notdef_rect,
        "\u{6587}\u{5b57}\u{5316}\u{3051} (Tofu): '标' and missing U+E000 share the same .notdef box {chinese_rect:?}. \
         Add Simplified-Chinese fallbacks to the Monospace family so source mode is readable."
    );
}
