/* WHY: Markdown/HTML links may address a heading inside the current or another document
 * (`notes.md#cap-004--queries`). The URL interceptor strips the fragment and opens the file;
 * this module resolves the fragment to a heading once the target preview pane exists. */

use crate::shell::{KatanaApp, PendingAnchorNavigation};
use katana_core::markdown::outline::HeadingAnchorOps;

/// WHY: A queued fragment is dropped if the target document never becomes available
/// (for example the linked file does not exist) so it cannot fire on a later navigation.
const PENDING_ANCHOR_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

pub(crate) struct AnchorNavigationOps;

impl AnchorNavigationOps {
    /// Splits `path#anchor` into its path and fragment parts.
    pub(crate) fn split_fragment(url: &str) -> (&str, Option<&str>) {
        match url.split_once('#') {
            Some((path, anchor)) if !anchor.is_empty() => (path, Some(anchor)),
            Some((path, _)) => (path, None),
            None => (url, None),
        }
    }

    pub(crate) fn queue(app: &mut KatanaApp, path: std::path::PathBuf, anchor: &str) {
        app.pending_anchor_navigation = Some(PendingAnchorNavigation {
            path,
            anchor: anchor.to_string(),
            queued_at: std::time::Instant::now(),
        });
    }

    pub(crate) fn clear(app: &mut KatanaApp) {
        app.pending_anchor_navigation = None;
    }

    /// Resolves a relative link target.
    ///
    /// Document-relative paths win, as in standard Markdown. Knowledge bases however often
    /// author links relative to the repository or workspace root, so the workspace root and
    /// (as a last resort) a workspace file whose path ends with the link are also tried.
    pub(crate) fn resolve_link_path(app: &KatanaApp, link_path: &str) -> std::path::PathBuf {
        let link = std::path::Path::new(link_path);
        if link.is_absolute() {
            return link.to_path_buf();
        }
        let document_relative = app
            .state
            .active_document()
            .and_then(|document| document.path.parent())
            .map(|parent| parent.join(link))
            .unwrap_or_else(|| link.to_path_buf());
        if document_relative.is_file() {
            return document_relative;
        }
        let Some(workspace) = app.state.workspace.data.as_ref() else {
            return document_relative;
        };
        let from_root = workspace.root.join(link);
        if from_root.is_file() {
            return from_root;
        }
        Self::find_workspace_file_by_suffix(&workspace.tree, link).unwrap_or(document_relative)
    }

    fn find_workspace_file_by_suffix(
        entries: &[katana_core::workspace::TreeEntry],
        link: &std::path::Path,
    ) -> Option<std::path::PathBuf> {
        let wanted: Vec<_> = link.components().collect();
        if wanted.is_empty() {
            return None;
        }
        let mut matches: Vec<std::path::PathBuf> = Vec::new();
        Self::collect_suffix_matches(entries, &wanted, &mut matches);
        /* WHY: Knowledge bases can hold the same relative path under several roots; the
         * shallowest match keeps the choice deterministic. */
        matches
            .into_iter()
            .min_by_key(|path| path.components().count())
    }

    fn collect_suffix_matches(
        entries: &[katana_core::workspace::TreeEntry],
        wanted: &[std::path::Component<'_>],
        matches: &mut Vec<std::path::PathBuf>,
    ) {
        for entry in entries {
            match entry {
                katana_core::workspace::TreeEntry::File { path } => {
                    if Self::ends_with_components(path, wanted) {
                        matches.push(path.clone());
                    }
                }
                katana_core::workspace::TreeEntry::Directory { children, .. } => {
                    Self::collect_suffix_matches(children, wanted, matches);
                }
            }
        }
    }

    fn ends_with_components(path: &std::path::Path, wanted: &[std::path::Component<'_>]) -> bool {
        let components: Vec<_> = path.components().collect();
        components.len() >= wanted.len() && components[components.len() - wanted.len()..] == *wanted
    }

    /// Turns a queued fragment into a preview scroll request.
    ///
    /// The request is only issued once the target document is open and its preview pane has
    /// been rendered; the scroll offset itself is resolved by the preview panel, which retries
    /// until the heading rect is known.
    pub(crate) fn apply_pending(app: &mut KatanaApp) {
        let Some(pending) = app.pending_anchor_navigation.take() else {
            return;
        };
        let is_open = app
            .state
            .document
            .open_documents
            .iter()
            .any(|document| document.path == pending.path);
        if !is_open {
            /* WHY: The document may still be loading (link clicked before the tab existed). */
            if pending.queued_at.elapsed() < PENDING_ANCHOR_TIMEOUT {
                app.pending_anchor_navigation = Some(pending);
            }
            return;
        }
        let Some(pane) = app
            .tab_previews
            .iter_mut()
            .find(|preview| preview.path == pending.path)
            .map(|preview| &mut preview.pane)
        else {
            return;
        };
        if let Some(index) =
            HeadingAnchorOps::find_heading_index(&pane.outline_items, &pending.anchor)
        {
            pane.scroll_request = Some(index);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::AnchorNavigationOps;
    use crate::app::DocumentOps;
    use crate::shell::KatanaApp;
    use std::sync::Arc;

    fn make_app() -> KatanaApp {
        let state = crate::app_state::AppState::new(
            katana_core::ai::AiProviderRegistry::new(),
            katana_core::plugin::PluginRegistry::new(),
            katana_platform::SettingsService::default(),
            Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        KatanaApp::new(state)
    }

    #[test]
    fn splits_markdown_link_into_path_and_anchor() {
        let (path, anchor) = AnchorNavigationOps::split_fragment(
            "draft/capability-catalog/UC_RLU_01_02_CCS2EVO.md#cap-004--queries",
        );

        assert_eq!(path, "draft/capability-catalog/UC_RLU_01_02_CCS2EVO.md");
        assert_eq!(anchor, Some("cap-004--queries"));
    }

    #[test]
    fn keeps_plain_paths_untouched() {
        let (path, anchor) = AnchorNavigationOps::split_fragment("notes.md");

        assert_eq!(path, "notes.md");
        assert_eq!(anchor, None);
    }

    #[test]
    fn treats_an_empty_fragment_as_a_plain_path() {
        let (path, anchor) = AnchorNavigationOps::split_fragment("notes.md#");

        assert_eq!(path, "notes.md");
        assert_eq!(anchor, None);
    }

    #[test]
    fn a_link_with_a_fragment_opens_the_file_and_scrolls_to_the_heading() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("UC_RLU_01_02_CCS2EVO.md");
        std::fs::write(
            &target,
            "# Overview\n\n## CAP-004 — Queries & Retrieves Remote Lock Action Execution Status\n",
        )
        .unwrap();
        let source = dir.path().join("index.md");
        std::fs::write(
            &source,
            "[link](UC_RLU_01_02_CCS2EVO.md#cap-004--queries--retrieves-remote-lock-action-execution-status)\n",
        )
        .unwrap();

        let mut app = make_app();
        app.handle_select_document(source.clone(), true);

        let (path_part, anchor) = AnchorNavigationOps::split_fragment(
            "UC_RLU_01_02_CCS2EVO.md#cap-004--queries--retrieves-remote-lock-action-execution-status",
        );
        let target_path = source.parent().unwrap().join(path_part);
        AnchorNavigationOps::queue(&mut app, target_path.clone(), anchor.unwrap());
        app.handle_select_document(target_path.clone(), true);
        AnchorNavigationOps::apply_pending(&mut app);

        let preview = app
            .tab_previews
            .iter()
            .find(|preview| preview.path == target_path)
            .expect("target preview pane");
        assert_eq!(preview.pane.scroll_request, Some(1));
        assert!(app.pending_anchor_navigation.is_none());
    }

    #[test]
    fn an_unknown_fragment_leaves_the_preview_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("notes.md");
        std::fs::write(&target, "# Overview\n\n## Details\n").unwrap();

        let mut app = make_app();
        app.handle_select_document(target.clone(), true);
        AnchorNavigationOps::queue(&mut app, target.clone(), "missing-heading");
        AnchorNavigationOps::apply_pending(&mut app);

        let preview = app
            .tab_previews
            .iter()
            .find(|preview| preview.path == target)
            .expect("preview pane");
        assert_eq!(preview.pane.scroll_request, None);
    }

    #[test]
    fn document_relative_targets_win_over_workspace_files() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("draft/capability-catalog");
        std::fs::create_dir_all(&nested).unwrap();
        let sibling = nested.join("notes.md");
        let root_relative = dir.path().join("notes.md");
        std::fs::write(&sibling, "# Sibling\n").unwrap();
        std::fs::write(&root_relative, "# Root\n").unwrap();
        let source = nested.join("index.md");
        std::fs::write(&source, "# Index\n").unwrap();

        let mut app = make_app();
        app.handle_select_document(source, true);

        assert_eq!(
            AnchorNavigationOps::resolve_link_path(&app, "notes.md"),
            sibling
        );
    }

    #[test]
    fn workspace_relative_targets_are_resolved_against_the_root() {
        let dir = tempfile::tempdir().unwrap();
        let catalog = dir.path().join("draft/capability-catalog");
        std::fs::create_dir_all(&catalog).unwrap();
        let target = catalog.join("UC_RLU_01_02_CCS2EVO.md");
        std::fs::write(&target, "# Overview\n").unwrap();
        let source = catalog.join("UC_RLU_01_01.md");
        std::fs::write(&source, "# Index\n").unwrap();

        let mut app = make_app();
        app.state.workspace.data = Some(katana_core::workspace::Workspace {
            root: dir.path().to_path_buf(),
            tree: vec![katana_core::workspace::TreeEntry::Directory {
                path: dir.path().join("draft"),
                children: vec![katana_core::workspace::TreeEntry::Directory {
                    path: catalog.clone(),
                    children: vec![katana_core::workspace::TreeEntry::File {
                        path: target.clone(),
                    }],
                }],
            }],
        });
        app.handle_select_document(source, true);

        assert_eq!(
            AnchorNavigationOps::resolve_link_path(
                &app,
                "draft/capability-catalog/UC_RLU_01_02_CCS2EVO.md"
            ),
            target
        );
    }

    #[test]
    fn link_targets_authored_from_the_repo_root_resolve_by_suffix() {
        let dir = tempfile::tempdir().unwrap();
        let catalog = dir.path().join("UID-27/UC_RLU_01/draft/capability-catalog");
        std::fs::create_dir_all(&catalog).unwrap();
        let target = catalog.join("UC_RLU_01_02_CCS2EVO.md");
        std::fs::write(&target, "# Overview\n").unwrap();
        let source = catalog.join("UC_RLU_01_01.md");
        std::fs::write(&source, "# Index\n").unwrap();

        let mut app = make_app();
        app.state.workspace.data = Some(katana_core::workspace::Workspace {
            root: dir.path().to_path_buf(),
            tree: vec![katana_core::workspace::TreeEntry::Directory {
                path: dir.path().join("UID-27/UC_RLU_01/draft"),
                children: vec![katana_core::workspace::TreeEntry::Directory {
                    path: catalog.clone(),
                    children: vec![katana_core::workspace::TreeEntry::File {
                        path: target.clone(),
                    }],
                }],
            }],
        });
        app.handle_select_document(source, true);

        assert_eq!(
            AnchorNavigationOps::resolve_link_path(
                &app,
                "draft/capability-catalog/UC_RLU_01_02_CCS2EVO.md"
            ),
            target
        );
    }
}
