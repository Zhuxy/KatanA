use super::types::*;
use crate::app_state::ScrollSource;
use eframe::egui;

impl EditorLogicOps {
    pub fn handle_scroll_to_line(
        ui: &mut egui::Ui,
        scroll: &mut crate::app_state::ScrollState,
        buffer: &str,
        response: &egui::Response,
        galley: &egui::text::Galley,
    ) {
        if let Some(target_line) = scroll.toc_scroll_to_line {
            Self::scroll_to_line_with_align(
                ui,
                scroll,
                buffer,
                target_line,
                response,
                galley,
                Self::jump_target_toc_scroll_rect,
            );
            return;
        }

        if let Some(target_line) = scroll.scroll_to_line {
            if scroll.last_scroll_to_line == Some(target_line) {
                return;
            }
            scroll.last_scroll_to_line = Some(target_line);
            Self::scroll_to_line_with_align(
                ui,
                scroll,
                buffer,
                target_line,
                response,
                galley,
                Self::jump_target_scroll_rect,
            );
        }
    }

    pub(crate) fn jump_target_scroll_rect(
        response_rect: &egui::Rect,
        cursor_rect: egui::Rect,
    ) -> (egui::Rect, egui::Align) {
        Self::jump_target_scroll_rect_with_align(response_rect, cursor_rect, egui::Align::Center)
    }

    pub(crate) fn jump_target_toc_scroll_rect(
        response_rect: &egui::Rect,
        cursor_rect: egui::Rect,
    ) -> (egui::Rect, egui::Align) {
        Self::jump_target_scroll_rect_with_align(response_rect, cursor_rect, egui::Align::TOP)
    }

    #[allow(clippy::too_many_arguments)]
    fn scroll_to_line_with_align(
        ui: &mut egui::Ui,
        scroll: &mut crate::app_state::ScrollState,
        buffer: &str,
        target_line: usize,
        response: &egui::Response,
        galley: &egui::text::Galley,
        target_outer_rect: fn(&egui::Rect, egui::Rect) -> (egui::Rect, egui::Align),
    ) {
        if let Some(idx) = Self::line_to_char_index(buffer, target_line) {
            let cursor = egui::text::CCursor {
                index: egui::text::CharIndex(idx),
                prefer_next_row: false,
            };
            let pos = galley.pos_from_cursor(cursor);
            let (rect, align) = target_outer_rect(&response.rect, pos);
            ui.scroll_to_rect(rect, Some(align));
            /* WHY: The resulting offset is self-inflicted, not a user scroll. Record it as an
             * echo on the next frame so the editor does not report it back to the preview. */
            Self::arm_jump_echo(scroll);
        }
    }

    /// Marks that the next editor offset update comes from a jump request.
    fn arm_jump_echo(scroll: &mut crate::app_state::ScrollState) {
        scroll.editor_jump_echo_pending = true;
    }

    fn jump_target_scroll_rect_with_align(
        response_rect: &egui::Rect,
        cursor_rect: egui::Rect,
        align: egui::Align,
    ) -> (egui::Rect, egui::Align) {
        (
            egui::Rect::from_min_max(
                egui::pos2(response_rect.min.x, response_rect.min.y + cursor_rect.min.y),
                egui::pos2(response_rect.max.x, response_rect.min.y + cursor_rect.max.y),
            ),
            align,
        )
    }

    pub fn update_scroll_sync(
        scroll: &mut crate::app_state::ScrollState,
        content_height: f32,
        inner_height: f32,
        offset_y: f32,
        consuming_preview: bool,
        dead_zone: f32,
        anchors: Vec<f32>,
    ) {
        let max_scroll = (content_height - inner_height).max(0.0);
        scroll.editor_max = max_scroll;
        scroll.editor_y = offset_y;
        scroll.editor_line_anchors = anchors;

        if scroll.editor_jump_echo_pending {
            scroll.editor_jump_echo_pending = false;
            scroll.editor_echo = crate::state::scroll_sync::SyncEcho { offset_y };
        }

        if consuming_preview {
            scroll.source = crate::app_state::ScrollSource::Neither;
            return;
        }
        if max_scroll <= 0.0 || scroll.editor_echo.is_echo(offset_y) {
            return;
        }
        if scroll.scroll_to_line.is_some() || scroll.toc_scroll_to_line.is_some() {
            return;
        }

        let current_logical = scroll.logical_position;
        let next_logical = scroll.mapper.editor_to_logical(offset_y);
        let dist = if next_logical.segment_index == current_logical.segment_index {
            (next_logical.progress - current_logical.progress).abs()
        } else {
            1.0
        };

        if dist > dead_zone {
            scroll.logical_position = next_logical;
            scroll.source = ScrollSource::Editor;
        }
    }
}

#[cfg(test)]
#[path = "logic_scroll_tests.rs"]
mod logic_scroll_tests;
