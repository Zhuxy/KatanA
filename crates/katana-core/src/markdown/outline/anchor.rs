/* WHY: Markdown links such as `other.md#cap-004--queries` carry a heading fragment that
 * must be resolved to a heading index. The slug rules mirror comrak's HTML anchorizer
 * (`comrak::html::anchorizer`) so links authored against KatanA's HTML export, GitHub, or
 * GitHub-compatible tools all resolve to the same heading. */

use super::types::OutlineItem;

/// Length of a percent escape (`%XX`) and of the two hexadecimal digits that follow it.
const PERCENT_ESCAPE_LEN: usize = 3;
/// Radix of the hexadecimal digits in a percent escape.
const HEX_RADIX: u32 = 16;

pub struct HeadingAnchorOps;

impl HeadingAnchorOps {
    /// Finds the heading index addressed by a URL fragment (`cap-004--queries-and-...`).
    ///
    /// Matching is case-insensitive, tolerates percent-encoding, and accepts a
    /// `user-content-` prefix emitted by sanitized HTML exports.
    pub fn find_heading_index(items: &[OutlineItem], anchor: &str) -> Option<usize> {
        let raw = Self::normalize_anchor(anchor);
        if raw.is_empty() {
            return None;
        }
        let needles = [Self::slugify(&raw), Self::loose_slug(&raw)];
        let mut used: Vec<String> = Vec::new();
        for item in items {
            let slug = Self::deduplicate(&Self::slugify(&item.text), &mut used);
            if needles.contains(&slug) || needles.contains(&Self::loose_slug(&item.text)) {
                return Some(item.index);
            }
        }
        None
    }

    /// Normalizes a URL fragment by percent-decoding it and dropping a sanitizer prefix.
    pub fn normalize_anchor(anchor: &str) -> String {
        let decoded = Self::percent_decode(anchor);
        let trimmed = decoded.trim().trim_start_matches('#');
        trimmed
            .strip_prefix("user-content-")
            .unwrap_or(trimmed)
            .to_string()
    }

    /// comrak-compatible heading slug: lowercase, drop everything but
    /// letters/marks/numbers/`-`/`_`/space, then turn spaces into dashes.
    pub fn slugify(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .filter(|c| Self::is_permitted(*c))
            .map(|c| if c == ' ' { '-' } else { c })
            .collect()
    }

    /// Fallback slug that keeps heading text comparable when the producer of the link
    /// used a different punctuation policy (for example `:` -> `-` instead of dropping it).
    fn loose_slug(text: &str) -> String {
        text.trim()
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '-'
                }
            })
            .collect()
    }

    fn deduplicate(slug: &str, used: &mut Vec<String>) -> String {
        let mut candidate = slug.to_string();
        let mut counter = 1usize;
        while used.contains(&candidate) {
            candidate = format!("{slug}-{counter}");
            counter += 1;
        }
        used.push(candidate.clone());
        candidate
    }

    /// Mirrors comrak's permitted set (`is_letter | is_mark | is_number | connector | ' ' | '-'`).
    /// Unicode mark categories are not exposed by `std`, so the common combining ranges are
    /// covered explicitly; without them decomposed accents would produce a different slug.
    fn is_permitted(character: char) -> bool {
        character == ' '
            || character == '-'
            || character == '_'
            || character.is_alphanumeric()
            || Self::is_combining_mark(character)
    }

    fn is_combining_mark(character: char) -> bool {
        matches!(character as u32,
            0x0300..=0x036F | 0x0483..=0x0489 | 0x0591..=0x05BD | 0x0610..=0x061A
            | 0x064B..=0x065F | 0x0670 | 0x06D6..=0x06DC | 0x0E31 | 0x0E34..=0x0E3A
            | 0x0E47..=0x0E4E | 0x20D0..=0x20F0 | 0xFE00..=0xFE0F)
    }

    fn percent_decode(input: &str) -> String {
        if !input.contains('%') {
            return input.to_string();
        }
        let bytes = input.as_bytes();
        let mut decoded: Vec<u8> = Vec::with_capacity(bytes.len());
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'%' && index + PERCENT_ESCAPE_LEN <= bytes.len() {
                let hex = std::str::from_utf8(&bytes[index + 1..index + PERCENT_ESCAPE_LEN]).ok();
                if let Some(byte) = hex.and_then(|value| u8::from_str_radix(value, HEX_RADIX).ok())
                {
                    decoded.push(byte);
                    index += PERCENT_ESCAPE_LEN;
                    continue;
                }
            }
            decoded.push(bytes[index]);
            index += 1;
        }
        String::from_utf8(decoded).unwrap_or_else(|_| input.to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn items(headings: &[&str]) -> Vec<OutlineItem> {
        headings
            .iter()
            .enumerate()
            .map(|(index, text)| OutlineItem {
                level: 2,
                text: (*text).to_string(),
                index,
                line_start: index,
                line_end: index,
            })
            .collect()
    }

    #[test]
    fn resolves_anchor_produced_by_the_html_exporter() {
        let outline = items(&[
            "Overview",
            "CAP-004 — Queries & Retrieves Remote Lock Action Execution Status from Kamereon",
        ]);

        assert_eq!(
            HeadingAnchorOps::find_heading_index(
                &outline,
                "cap-004--queries--retrieves-remote-lock-action-execution-status-from-kamereon"
            ),
            Some(1)
        );
    }

    #[test]
    fn resolves_anchor_written_with_punctuation_dropped() {
        let outline = items(&["Installation & Setup"]);

        assert_eq!(
            HeadingAnchorOps::find_heading_index(&outline, "installation--setup"),
            Some(0)
        );
    }

    #[test]
    fn duplicate_heading_text_resolves_to_the_numbered_anchor() {
        let outline = items(&["Notes", "Notes"]);

        assert_eq!(
            HeadingAnchorOps::find_heading_index(&outline, "notes"),
            Some(0)
        );
        assert_eq!(
            HeadingAnchorOps::find_heading_index(&outline, "notes-1"),
            Some(1)
        );
    }

    #[test]
    fn tolerates_percent_encoding_and_sanitizer_prefix() {
        let outline = items(&["設計方針-概要"]);

        assert_eq!(
            HeadingAnchorOps::find_heading_index(
                &outline,
                "user-content-%E8%A8%AD%E8%A8%88%E6%96%B9%E9%87%9D-%E6%A6%82%E8%A6%81"
            ),
            Some(0)
        );
    }

    #[test]
    fn falls_back_to_loose_match_for_alternative_punctuation() {
        let outline = items(&["CAP-004: Queries & Retrieves"]);

        /* WHY: `:` becomes `-` here instead of being dropped, as in github-slugger style tools. */
        assert_eq!(
            HeadingAnchorOps::find_heading_index(&outline, "cap-004--queries---retrieves"),
            Some(0)
        );
    }

    #[test]
    fn unknown_and_empty_anchors_resolve_to_nothing() {
        let outline = items(&["Overview"]);

        assert_eq!(
            HeadingAnchorOps::find_heading_index(&outline, "missing"),
            None
        );
        assert_eq!(HeadingAnchorOps::find_heading_index(&outline, "#"), None);
    }
}
