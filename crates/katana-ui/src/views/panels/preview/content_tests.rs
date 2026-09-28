/* WHY: The table of contents and `#anchor` link navigation both arm a *sticky* heading jump
 * in the preview. These tests pin the two properties that make it reliable: the jump survives
 * later frames that would otherwise move the preview (an editor-driven scroll-sync offset), and
 * it is released as soon as the user scrolls. */

use super::types::PreviewContent;
use crate::app_state::{AppAction, ScrollSource, ScrollState};
use crate::preview_pane::PreviewPane;
use crate::preview_pane::heading_jump::HeadingJumpOps;
use crate::views::panels::editor::types::EditorLogicOps;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

/// The H1 occupies heading index 0, so `## Section N` sits at index `N + 1`.
const SECTION_12_INDEX: usize = 13;

const PREVIEW_WIDTH: f32 = 600.0;
const PREVIEW_HEIGHT: f32 = 400.0;

fn long_markdown() -> String {
    let mut md = String::from("# Doc Title\n\n");
    for i in 0..20 {
        md.push_str(&format!("## Section {i}\n\n"));
        for j in 0..6 {
            md.push_str(&format!("paragraph {i}-{j} lorem ipsum dolor sit amet\n\n"));
        }
    }
    md
}

struct Fixture {
    harness: Harness<'static, ()>,
    preview: std::rc::Rc<std::cell::RefCell<PreviewPane>>,
    scroll: std::rc::Rc<std::cell::RefCell<ScrollState>>,
    path: std::path::PathBuf,
}

fn fixture(scroll_sync: bool) -> Fixture {
    let preview = std::rc::Rc::new(std::cell::RefCell::new(PreviewPane::default()));
    let scroll = std::rc::Rc::new(std::cell::RefCell::new(ScrollState::new()));
    let action = std::rc::Rc::new(std::cell::RefCell::new(AppAction::None));
    let path = std::path::PathBuf::from("/tmp/heading_jump_test.md");
    let document = katana_core::document::Document::new(path.clone(), long_markdown());

    let path_for_build = path.clone();
    let preview_for_build = preview.clone();
    let scroll_for_build = scroll.clone();
    let action_for_build = action.clone();
    let harness = Harness::builder()
        .with_size(egui::vec2(PREVIEW_WIDTH, PREVIEW_HEIGHT))
        .build_ui(move |ui| {
            let mut pane = preview_for_build.borrow_mut();
            if pane.sections.is_empty() {
                pane.full_render(
                    &long_markdown(),
                    &path_for_build,
                    std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
                    false,
                    4,
                );
                pane.wait_for_renders();
            }
            PreviewContent {
                preview: &mut pane,
                document: Some(&document),
                scroll: &mut scroll_for_build.borrow_mut(),
                action: &mut action_for_build.borrow_mut(),
                scroll_sync,
                search_query: None,
                doc_search_active_index: None,
            }
            .show(ui);
        });

    Fixture {
        harness,
        preview,
        scroll,
        path,
    }
}

/// Y position of the preview label that contains `text`.
fn label_y(fixture: &Fixture, text: &str) -> f32 {
    fixture
        .harness
        .query_by_label_contains(text)
        .expect("label is rendered")
        .rect()
        .min
        .y
}

impl Fixture {
    fn settle(&mut self, frames: usize) {
        for _ in 0..frames {
            self.harness.step();
        }
    }

    fn arm(&self, index: usize) {
        HeadingJumpOps::request(&mut self.preview.borrow_mut(), index);
    }

    fn armed(&self) -> bool {
        HeadingJumpOps::is_armed(&self.preview.borrow())
    }
}

#[test]
fn heading_jump_puts_the_target_at_the_top_of_the_viewport() {
    let mut fixture = fixture(false);
    fixture.settle(3);

    fixture.arm(SECTION_12_INDEX);
    fixture.settle(3);

    let y = label_y(&fixture, "Section 12");
    assert!(
        (y - 4.0).abs() < 12.0,
        "the armed heading should sit at the top of the preview, got y={y}"
    );
    assert!(
        fixture.armed(),
        "the jump must stay armed after it is applied"
    );
}

#[test]
fn the_editor_scroll_triggered_by_the_jump_keeps_the_jump_armed() {
    let mut fixture = fixture(true);
    fixture.settle(3);

    fixture.arm(SECTION_12_INDEX);
    fixture.settle(3);
    let after_jump = label_y(&fixture, "Section 12");

    /* WHY: In split view the editor also follows the jump. Its offset is recorded as an echo
     * (see `EditorLogicOps::update_scroll_sync`) so it is not reported back as a user scroll,
     * which previously overwrote the preview position with the editor's old one. */
    fixture.scroll.borrow_mut().editor_jump_echo_pending = true;
    EditorLogicOps::update_scroll_sync(
        &mut fixture.scroll.borrow_mut(),
        2000.0,
        400.0,
        900.0,
        false,
        0.02,
        vec![0.0; 200],
    );
    fixture.settle(3);

    assert_ne!(
        fixture.scroll.borrow().source,
        ScrollSource::Editor,
        "a jump-driven editor offset must not be reported as a user scroll"
    );
    assert_eq!(
        after_jump,
        label_y(&fixture, "Section 12"),
        "an editor-driven sync must not undo an armed heading jump"
    );
    assert!(fixture.armed());
}

#[test]
fn a_user_scroll_in_the_editor_releases_the_armed_heading_jump() {
    let mut fixture = fixture(true);
    fixture.settle(3);

    fixture.arm(SECTION_12_INDEX);
    fixture.settle(3);
    assert!(fixture.armed());

    fixture.scroll.borrow_mut().source = ScrollSource::Editor;
    fixture.scroll.borrow_mut().logical_position =
        crate::state::scroll_sync::LogicalPosition::default();
    fixture.settle(2);

    assert!(
        !fixture.armed(),
        "once the user scrolls the editor, the preview must follow the sync again"
    );
}

#[test]
fn user_scrolling_releases_the_armed_heading_jump() {
    let mut fixture = fixture(false);
    fixture.settle(3);

    fixture.arm(SECTION_12_INDEX);
    fixture.settle(3);
    let after_jump = label_y(&fixture, "Section 12");

    /* WHY: egui only applies wheel scrolling while the pointer is over the scroll area. */
    fixture
        .harness
        .hover_at(egui::pos2(PREVIEW_WIDTH / 2.0, PREVIEW_HEIGHT / 2.0));
    fixture.settle(2);
    fixture
        .harness
        .input_mut()
        .events
        .push(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, 240.0),
            phase: egui::TouchPhase::Move,
            modifiers: egui::Modifiers::default(),
        });
    fixture.settle(3);

    assert!(
        !fixture.armed(),
        "a wheel scroll must release the heading jump"
    );
    let after_wheel = label_y(&fixture, "Section 12");
    assert!(
        (after_wheel - after_jump).abs() > 20.0,
        "the preview must follow the wheel after the jump is released, \
         after_jump={after_jump} after_wheel={after_wheel}"
    );
}

#[test]
fn a_new_document_render_drops_the_armed_jump() {
    let mut fixture = fixture(false);
    fixture.settle(3);
    fixture.arm(SECTION_12_INDEX);
    assert!(fixture.armed());

    fixture.preview.borrow_mut().full_render(
        &long_markdown(),
        &fixture.path,
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        false,
        4,
    );

    assert!(
        !fixture.armed(),
        "a re-rendered source invalidates the heading rects, so the jump must be dropped"
    );
}

#[test]
fn a_search_jump_takes_over_the_armed_heading_jump() {
    let mut fixture = fixture(false);
    fixture.settle(3);
    fixture.arm(SECTION_12_INDEX);
    fixture.settle(2);

    fixture.scroll.borrow_mut().scroll_to_line = Some(2);
    fixture.settle(2);

    assert!(
        !fixture.armed(),
        "another scroll driver must take precedence over the armed heading jump"
    );
}
