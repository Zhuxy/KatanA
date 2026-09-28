/* WHY: The table of contents and `#anchor` link navigation both need to put a heading at the
 * top of the preview viewport. Applying a single one-shot offset is not enough: the offset is
 * derived from the previous frame's rects, so any later layout change (a floating panel
 * closing, a resize, a sync write from the editor) can drop or displace the jump — the preview
 * then silently returns to where it was. A *sticky* target re-derives the offset every frame
 * and only yields when the user scrolls or another scroll driver takes over. */

use super::types::PreviewPane;

pub struct HeadingJumpOps;

/// Tolerance in pixels between the requested and the applied scroll offset. Anything larger
/// means the scroll area settled somewhere else, i.e. the user scrolled.
const APPLIED_OFFSET_EPSILON: f32 = 2.0;

impl HeadingJumpOps {
    /// Arms a sticky jump to a heading index (`0` means "very top of the document").
    pub fn request(pane: &mut PreviewPane, index: usize) {
        pane.scroll_request = Some(index);
        pane.heading_jump_applied = None;
    }

    /// Releases the armed jump; scrolling returns to the user and the editor sync.
    pub fn release(pane: &mut PreviewPane) {
        pane.scroll_request = None;
        pane.heading_jump_applied = None;
    }

    pub fn is_armed(pane: &PreviewPane) -> bool {
        pane.scroll_request.is_some()
    }

    /// Records the offset forced for the armed target this frame.
    pub fn note_applied(pane: &mut PreviewPane, index: usize, offset: f32) {
        pane.scroll_request = Some(index);
        pane.heading_jump_applied = Some(offset);
    }

    /// Offset that puts the armed heading at the top of the viewport.
    ///
    /// Returns `None` while the heading rect is unknown (first frame of a new document), which
    /// keeps the jump armed so the following frame can retry.
    pub fn resolve_offset(pane: &mut PreviewPane, content_top_y: f32) -> Option<(usize, f32)> {
        let index = pane.scroll_request?;
        let offset = if index == 0 {
            0.0
        } else {
            Self::scroll_offset_for(index, &pane.anchor_map, content_top_y)?
        };
        Some((index, offset))
    }

    /// Precise preview offset that puts a heading at the top of the viewport, derived from the
    /// previous frame's heading rects (screen space) relative to the content origin.
    pub fn scroll_offset_for(
        heading_index: usize,
        anchor_map: &[super::types::DocumentAnchorMapItem],
        content_top_y: f32,
    ) -> Option<f32> {
        let item = anchor_map.iter().find(|a| a.index == Some(heading_index))?;
        let rect = item.outer_rect?;
        /* WHY: rect.min.y is screen-space. Subtracting content_top_y converts it to the
         * ScrollArea's virtual-space offset (offset = 0 at the very top of the content, before
         * any Frame/padding offset). Negative values are clamped so the first heading stays
         * reachable. */
        Some((rect.min.y - content_top_y).max(0.0))
    }

    /// Called after the scroll area ran: records what was forced and releases the jump when
    /// the applied offset no longer matches (the user scrolled) or when another driver such as
    /// a search jump took over.
    pub fn settle(
        pane: &mut PreviewPane,
        applied_offset: Option<f32>,
        content_height: f32,
        inner_height: f32,
        user_scrolled: bool,
        other_driver_active: bool,
    ) {
        if !Self::is_armed(pane) {
            return;
        }
        if user_scrolled || other_driver_active {
            Self::release(pane);
            return;
        }
        let Some(applied_offset) = applied_offset else {
            /* WHY: The scroll area was not rendered this frame (for example a document
             * surface took over). Keep the jump armed and retry next frame. */
            return;
        };
        let max_offset = (content_height - inner_height).max(0.0);
        let expected = pane
            .heading_jump_applied
            .map(|offset| offset.clamp(0.0, max_offset))
            .unwrap_or(applied_offset);
        if (applied_offset - expected).abs() > APPLIED_OFFSET_EPSILON {
            Self::release(pane);
            return;
        }
        pane.heading_jump_applied = Some(expected);
    }
}
