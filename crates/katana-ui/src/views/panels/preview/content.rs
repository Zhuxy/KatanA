mod constructor;

use super::content_jump::PreviewJumpOps;
use super::types::*;
use super::{content_html_browser::show_html_browser_content, types::PreviewLogicOps};
use crate::app_state::AppAction;
use eframe::egui;

const BACK_TO_TOP_THRESHOLD: f32 = 400.0;

/// Id salt of the preview scroll area; the scrollbar lives under this id.
pub(super) const PREVIEW_SCROLL_AREA_ID: &str = "preview_scroll_area";

impl<'a> PreviewContent<'a> {
    pub fn show(self, ui: &mut egui::Ui) {
        let PreviewContent {
            preview,
            document,
            scroll,
            action,
            scroll_sync,
            search_query,
            doc_search_active_index,
        } = self;

        /* WHY: Lock preview content to the panel's current available width so
         * intrinsic-size widgets cannot expand the parent resizable panel state. */
        let panel_width = ui.available_width();
        ui.set_min_width(panel_width);
        ui.set_max_width(panel_width);

        if preview.has_document_surface() {
            preview.show_content(
                ui,
                scroll.active_editor_line,
                None,
                search_query,
                doc_search_active_index,
            );
            return;
        }

        if preview.has_html_browser() {
            show_html_browser_content(
                preview,
                ui,
                scroll.active_editor_line,
                search_query,
                doc_search_active_index,
                action,
            );
            return;
        }

        /* WHY: Check for a forced scroll target from the sync system, the table of contents or
         * an `#anchor` link navigation. */
        let (forced_offset, editor_drove_scroll) =
            PreviewJumpOps::apply(ui, scroll_sync, scroll, preview);

        let mut scroll_area = egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .id_salt((
                PREVIEW_SCROLL_AREA_ID,
                document.map(|doc| doc.path.as_path()),
            ));

        if let Some(offset) = forced_offset {
            scroll_area = scroll_area.vertical_scroll_offset(offset);
        }

        /* WHY: Keep the scroll area itself full-width so its scrollbar touches the
         * preview edge; width capping still happens through this fixed child rect. */
        let inner_content_width = ui.available_width();
        let child_rect = egui::Rect::from_min_size(
            ui.next_widget_position(),
            egui::vec2(inner_content_width, ui.available_height()),
        );

        let output = ui
            .scope_builder(
                egui::UiBuilder::new()
                    .max_rect(child_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
                |ui| {
                    scroll_area.show(ui, |ui| {
                        /* WHY: Use a Frame with explicit horizontal margin for consistent padding (12px). */
                        egui::Frame::NONE
                            .inner_margin(egui::Margin::symmetric(
                                crate::shell_ui::PREVIEW_CONTENT_PADDING,
                                0,
                            ))
                            .show(ui, |ui| {
                                PreviewLogicOps::render_preview_top_padding(ui);
                                let is_interactive = ui.is_enabled();
                                let mut hovered_lines = Vec::new();
                                let hover_out = if is_interactive {
                                    Some(&mut hovered_lines)
                                } else {
                                    None
                                };

                                let actions = preview.show_content(
                                    ui,
                                    scroll.active_editor_line,
                                    hover_out,
                                    search_query.clone(),
                                    doc_search_active_index,
                                );

                                /* WHY: Hover synchronization is decoupled from scroll_sync.        */
                                if is_interactive {
                                    scroll.hovered_preview_lines = hovered_lines;
                                }

                                /* WHY: Handle clicks for Editor-to-Preview navigation. */
                                if is_interactive
                                    && !preview.html_browser_is_interacting()
                                    && ui.rect_contains_pointer(ui.min_rect())
                                    && ui.input(|i| i.pointer.primary_clicked())
                                    && let Some(hovered) = scroll.hovered_preview_lines.first()
                                {
                                    scroll.scroll_to_line = Some(hovered.start);
                                }

                                /* WHY: Handle embedded actions like Task List toggling. */
                                if let Some((global_index, new_state)) = actions.into_iter().next() {
                                    *action = AppAction::ToggleTaskList {
                                        global_index,
                                        new_state,
                                    };
                                }

                                PreviewLogicOps::render_preview_bottom_padding(ui, scroll);
                            });
                    })
                },
            );

        if scroll_sync {
            PreviewLogicOps::update_scroll_sync(
                scroll,
                preview,
                crate::shell::TREE_ROW_HEIGHT,
                output.inner.content_size.y,
                output.inner.inner_rect.height(),
                output.inner.state.offset.y,
            );
        }

        PreviewJumpOps::settle(
            ui,
            preview,
            scroll,
            output.inner.state.offset.y,
            output.inner.content_size.y,
            output.inner.inner_rect.height(),
            document.map(|doc| doc.path.as_path()),
            editor_drove_scroll,
        );

        /* WHY: We no longer need PreviewHeader. The TOC is toggled from the AppFrame side panel.
        Export and Story view are now floating buttons at the bottom right. */

        /* WHY: Render floating action buttons at the bottom right. */
        let scroll_offset = output.inner.state.offset.y;
        let show_back_to_top = scroll_offset > BACK_TO_TOP_THRESHOLD;
        PreviewLogicOps::render_floating_buttons(
            ui,
            document.is_some(),
            show_back_to_top,
            action,
            preview,
        );

        /* WHY: FB25 — For LinterDocs, render a "View on GitHub" button at the
         * top-right inside the preview pane (not in the toolbar). */
        if let Some(doc) = document {
            PreviewLogicOps::render_linter_docs_github_button(ui, &doc.path);
        }
    }
}
