/* WHY: The preview panel must keep an armed heading jump (table of contents or `#anchor` link)
 * pinned to its heading. Both halves of that live here so `content.rs` only orchestrates:
 * `apply` decides the forced offset for the frame, `settle` decides when the jump gives way. */

use super::types::PreviewLogicOps;
use crate::app_state::ScrollState;
use crate::preview_pane::PreviewPane;
use crate::preview_pane::heading_jump::HeadingJumpOps;
use eframe::egui;

/// A wheel delta below this is treated as no scroll at all (trackpad noise).
const WHEEL_SCROLL_EPSILON: f32 = 0.01;

pub struct PreviewJumpOps;

impl PreviewJumpOps {
    /// Offset the preview must show this frame, combining the scroll sync with the armed jump.
    ///
    /// An armed jump outranks the sync and is re-derived from the current heading rects, so a
    /// later layout change cannot silently drop it. Returns the sync source as seen *before*
    /// the scroll area runs, because `update_scroll_sync` consumes it at the end of the frame.
    pub fn apply(
        ui: &egui::Ui,
        scroll_sync: bool,
        scroll: &mut ScrollState,
        preview: &mut PreviewPane,
    ) -> (Option<f32>, bool) {
        let editor_drove_scroll = scroll_sync
            && scroll.source == crate::app_state::ScrollSource::Editor
            && scroll.scroll_to_line.is_none();

        let mut forced_offset = PreviewLogicOps::compute_forced_offset(
            scroll_sync,
            scroll,
            preview,
            crate::shell::TREE_ROW_HEIGHT,
            ui.available_height(),
        );

        if let Some((index, offset)) =
            HeadingJumpOps::resolve_offset(preview, preview.content_top_y)
        {
            forced_offset = Some(offset);
            HeadingJumpOps::note_applied(preview, index, offset);
        }

        (forced_offset, editor_drove_scroll)
    }

    /// Releases the armed heading jump once the user scrolls (wheel or scrollbar drag) or once
    /// another scroll driver (a search jump, a real editor scroll) takes over.
    #[allow(clippy::too_many_arguments)]
    pub fn settle(
        ui: &egui::Ui,
        preview: &mut PreviewPane,
        scroll: &ScrollState,
        applied_offset: f32,
        content_height: f32,
        inner_height: f32,
        document_path: Option<&std::path::Path>,
        editor_drove_scroll: bool,
    ) {
        if !HeadingJumpOps::is_armed(preview) {
            return;
        }
        let user_scrolled = ui.input(|i| i.smooth_scroll_delta.y.abs() > WHEEL_SCROLL_EPSILON)
            || Self::scrollbar_dragged(ui.ctx(), document_path);
        let other_driver_active = scroll.scroll_to_line.is_some()
            || scroll.toc_scroll_to_line.is_some()
            || editor_drove_scroll;
        HeadingJumpOps::settle(
            preview,
            Some(applied_offset),
            content_height,
            inner_height,
            user_scrolled,
            other_driver_active,
        );
    }

    /// WHY: A scrollbar drag changes the offset without any wheel event, so it needs its own check.
    fn scrollbar_dragged(ctx: &egui::Context, path: Option<&std::path::Path>) -> bool {
        let Some(path) = path else {
            return false;
        };
        ctx.dragged_id()
            == Some(egui::Id::new((
                super::content::PREVIEW_SCROLL_AREA_ID,
                path,
            )))
    }
}
