use super::side_panel_toggle::decorate_toggle_response;
use super::side_panels::{LIGHT_MODE_ICON_BG, PreviewSidePanels};
use crate::app_state::{AppAction, ViewMode};
use crate::icon::IconSize;
use eframe::egui;

const SEGMENT_SIZE: f32 = 28.0;
const SEGMENT_ROUNDING: u8 = 4;
const LIGHT_MODE_ACTIVE_BG: u8 = 230;
const GROUP_BOTTOM_SPACE: f32 = 8.0;

impl<'a> PreviewSidePanels<'a> {
    /// Render the always-visible three-state view mode toggle (Preview / Code / Split).
    ///
    /// WHY: Keeping the selector directly on the sidebar removes the need to open the
    /// tools hover popup just to change the document layout.
    pub(super) fn render_view_mode_selector(&mut self, ui: &mut egui::Ui) {
        let i18n = crate::i18n::I18nOps::get();
        let current = self.app.state.active_view_mode();
        let segments = [
            (
                ViewMode::PreviewOnly,
                crate::Icon::Document,
                i18n.view_mode.preview.as_str(),
            ),
            (
                ViewMode::CodeOnly,
                crate::Icon::Code,
                i18n.view_mode.code.as_str(),
            ),
            (
                ViewMode::Split,
                crate::Icon::SplitVertical,
                i18n.view_mode.split.as_str(),
            ),
        ];

        let mut requested_mode = None;
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for (index, (mode, icon, label)) in segments.iter().enumerate() {
                let is_active = *mode == current;
                if view_mode_segment(ui, *icon, label, is_active, index, segments.len()) {
                    requested_mode = Some(*mode);
                }
            }
        });

        if let Some(mode) = requested_mode {
            self.app.pending_action = AppAction::SetViewMode(mode);
        }
        ui.add_space(GROUP_BOTTOM_SPACE);
    }
}

/* WHY: Module-private composition helper keeps the rendering loop readable. */
fn view_mode_segment(
    ui: &mut egui::Ui,
    icon: crate::Icon,
    label: &str,
    is_active: bool,
    index: usize,
    segment_count: usize,
) -> bool {
    let corner_radius = egui::CornerRadius {
        nw: rounded_first(index),
        ne: rounded_first(index),
        sw: rounded_last(index, segment_count),
        se: rounded_last(index, segment_count),
    };
    #[rustfmt::skip]
    let inactive_fill = if ui.visuals().dark_mode { crate::theme_bridge::TRANSPARENT } else { crate::theme_bridge::ThemeBridgeOps::from_gray(LIGHT_MODE_ICON_BG) };
    #[rustfmt::skip]
    let active_fill = if ui.visuals().dark_mode { ui.visuals().selection.bg_fill } else { crate::theme_bridge::ThemeBridgeOps::from_gray(LIGHT_MODE_ACTIVE_BG) };

    let response = ui.add(
        egui::Button::image(icon.ui_image(ui, IconSize::Medium))
            .fill(if is_active {
                active_fill
            } else {
                inactive_fill
            })
            .min_size(egui::vec2(SEGMENT_SIZE, SEGMENT_SIZE))
            .corner_radius(corner_radius),
    );
    decorate_toggle_response(ui, response, label, None).clicked()
}

fn rounded_first(index: usize) -> u8 {
    if index == 0 { SEGMENT_ROUNDING } else { 0 }
}

fn rounded_last(index: usize, segment_count: usize) -> u8 {
    if index + 1 == segment_count {
        SEGMENT_ROUNDING
    } else {
        0
    }
}
