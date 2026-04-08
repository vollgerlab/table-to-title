use crate::parse::AuthorRow;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct FormatOptions {
    #[serde(default = "default_true")]
    pub show_corresponding: bool,
    #[serde(default = "default_true")]
    pub show_equal: bool,
    #[serde(default)]
    pub show_orcid: bool,
}

fn default_true() -> bool {
    true
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self { show_corresponding: true, show_equal: true, show_orcid: false }
    }
}

#[derive(Debug, Clone)]
pub struct Author {
    pub name: String,
    pub aff_numbers: Vec<usize>,
    pub is_corresponding: bool,
    pub is_equal: bool,
    pub orcid: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthorData {
    pub authors: Vec<Author>,
    /// (1-based number, affiliation text), sorted by number
    pub affiliations: Vec<(usize, String)>,
}

/// Build deduplicated affiliation index and author list from parsed rows.
pub fn build_author_data(rows: &[AuthorRow]) -> AuthorData {
    let mut aff_index: HashMap<String, usize> = HashMap::new();
    let mut aff_order: Vec<String> = Vec::new();

    for row in rows {
        for inst in &row.institutions {
            if !aff_index.contains_key(inst) {
                let num = aff_order.len() + 1;
                aff_index.insert(inst.clone(), num);
                aff_order.push(inst.clone());
            }
        }
    }

    let authors = rows
        .iter()
        .map(|row| {
            let name_parts: Vec<&str> = [
                row.first.as_str(),
                row.middle.as_str(),
                row.last.as_str(),
                row.suffix.as_str(),
            ]
            .iter()
            .copied()
            .filter(|s| !s.is_empty())
            .collect();
            let name = name_parts.join(" ");

            // Sort affiliation numbers ascending so they always display as 1,2 not 2,1
            let mut aff_numbers: Vec<usize> = row
                .institutions
                .iter()
                .filter_map(|inst| aff_index.get(inst).copied())
                .collect();
            aff_numbers.sort_unstable();

            let orcid = if row.orcid.is_empty() {
                None
            } else {
                Some(format!("https://orcid.org/{}", row.orcid))
            };

            Author {
                name,
                aff_numbers,
                is_corresponding: row.is_corresponding,
                is_equal: row.is_equal_contribution,
                orcid,
            }
        })
        .collect();

    let affiliations = aff_order
        .into_iter()
        .enumerate()
        .map(|(i, aff)| (i + 1, aff))
        .collect();

    AuthorData { authors, affiliations }
}

/// Superscript string for an author: [1,2,3][*][†]
fn sup_text(author: &Author, opts: &FormatOptions) -> String {
    let mut sup = String::new();
    if !author.aff_numbers.is_empty() {
        let nums = author.aff_numbers.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",");
        sup.push_str(&nums);
    }
    if opts.show_equal && author.is_equal {
        sup.push('*');
    }
    if opts.show_corresponding && author.is_corresponding {
        sup.push('\u{2020}');
    }
    sup
}

/// Legend lines to append after affiliations.
pub fn legend_lines(data: &AuthorData, opts: &FormatOptions) -> Vec<String> {
    let mut lines = Vec::new();
    if opts.show_equal && data.authors.iter().any(|a| a.is_equal) {
        lines.push("* These authors contributed equally to this work.".to_string());
    }
    if opts.show_corresponding && data.authors.iter().any(|a| a.is_corresponding) {
        lines.push("\u{2020}Corresponding author(s).".to_string());
    }
    lines
}

/// Author line as HTML with <sup> tags for affiliation numbers.
pub fn author_line_html(data: &AuthorData, opts: &FormatOptions) -> String {
    let parts: Vec<String> = data
        .authors
        .iter()
        .map(|a| {
            let mut s = html_escape(&a.name);
            let sup = sup_text(a, opts);
            if !sup.is_empty() {
                s.push_str(&format!("<sup>{}</sup>", sup));
            }
            s
        })
        .collect();
    parts.join(", ")
}

/// Author line as plain text with inline numbers (no HTML).
pub fn author_line_plain(data: &AuthorData, opts: &FormatOptions) -> String {
    let parts: Vec<String> = data
        .authors
        .iter()
        .map(|a| {
            let mut s = a.name.clone();
            s.push_str(&sup_text(a, opts));
            s
        })
        .collect();
    parts.join(", ")
}

/// Complete preview HTML fragment (title + authors + affiliations + legend + ORCiDs).
pub fn build_preview_html(data: &AuthorData, opts: &FormatOptions, title: &str) -> String {
    let mut html = String::new();

    if !title.trim().is_empty() {
        html.push_str(&format!(
            "<div class=\"preview-title\">{}</div>",
            html_escape(title.trim())
        ));
    }

    html.push_str(&format!("<div>{}</div>", author_line_html(data, opts)));

    html.push_str("<div class=\"aff-list\">");
    for (num, aff) in &data.affiliations {
        html.push_str(&format!("<p>{}. {}</p>", num, html_escape(aff)));
    }
    html.push_str("</div>");

    for legend in legend_lines(data, opts) {
        html.push_str(&format!("<p class=\"legend\">{}</p>", html_escape(&legend)));
    }

    if opts.show_orcid {
        let orcids: Vec<_> = data
            .authors
            .iter()
            .filter_map(|a| a.orcid.as_ref().map(|o| (a.name.as_str(), o.as_str())))
            .collect();
        if !orcids.is_empty() {
            html.push_str("<div class=\"orcid-section\"><strong>ORCiD</strong>");
            for (name, orcid) in orcids {
                html.push_str(&format!(
                    "<p>{}: <a href=\"{}\" target=\"_blank\">{}</a></p>",
                    html_escape(name),
                    html_escape(orcid),
                    html_escape(orcid)
                ));
            }
            html.push_str("</div>");
        }
    }

    html
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Normalise an affiliation string for comparison: lowercase, collapse
/// internal whitespace, strip trailing punctuation.
pub fn normalize_aff(s: &str) -> String {
    let collapsed = s.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.trim_end_matches(|c: char| c == '.' || c == ',').trim().to_string()
}

/// Detect redundant affiliations and return `(drop, keep)` pairs.
///
/// Two tiers:
/// 1. Whitespace/case normalisation — two raw strings that normalise identically
///    are definite duplicates; keep the first occurrence.
/// 2. Substring containment — if the normalised form of A is wholly contained
///    inside the normalised form of B, A is the less-specific form; suggest
///    merging A → B (the longer, more specific string).
pub fn find_merge_suggestions(rows: &[AuthorRow]) -> Vec<(String, String)> {
    // Collect unique raw affiliation strings in order of first appearance.
    let mut seen: Vec<String> = Vec::new();
    for row in rows {
        for inst in &row.institutions {
            if !seen.contains(inst) {
                seen.push(inst.clone());
            }
        }
    }

    let normed: Vec<String> = seen.iter().map(|s| normalize_aff(s)).collect();
    let mut suggestions: Vec<(String, String)> = Vec::new();
    let mut already_dropped: std::collections::HashSet<usize> = Default::default();

    for i in 0..seen.len() {
        if already_dropped.contains(&i) {
            continue;
        }
        for j in (i + 1)..seen.len() {
            if already_dropped.contains(&j) {
                continue;
            }
            let ni = &normed[i];
            let nj = &normed[j];

            if ni == nj {
                // Tier 1: identical after normalisation — definite duplicate.
                // Keep the earlier occurrence (i), drop j.
                suggestions.push((seen[j].clone(), seen[i].clone()));
                already_dropped.insert(j);
            } else if nj.contains(ni.as_str()) {
                // i normalised is a substring of j normalised → i is less specific.
                suggestions.push((seen[i].clone(), seen[j].clone()));
                already_dropped.insert(i);
                break; // i is now scheduled to be dropped; skip further comparisons for i.
            } else if ni.contains(nj.as_str()) {
                // j normalised is a substring of i normalised → j is less specific.
                suggestions.push((seen[j].clone(), seen[i].clone()));
                already_dropped.insert(j);
            }
        }
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::AuthorRow;

    fn make_rows() -> Vec<AuthorRow> {
        vec![
            AuthorRow {
                email: "a@x.com".into(),
                first: "Alice".into(),
                middle: "B.".into(),
                last: "Smith".into(),
                suffix: "".into(),
                institutions: vec!["Dept A".into(), "Dept B".into()],
                is_corresponding: true,
                is_equal_contribution: true,
                orcid: "0000-0001-0000-0000".into(),
            },
            AuthorRow {
                email: "b@x.com".into(),
                first: "Bob".into(),
                middle: "".into(),
                last: "Jones".into(),
                suffix: "".into(),
                institutions: vec!["Dept B".into()],
                is_corresponding: false,
                is_equal_contribution: false,
                orcid: "".into(),
            },
        ]
    }

    #[test]
    fn affiliation_dedup_order() {
        let rows = make_rows();
        let data = build_author_data(&rows);
        assert_eq!(data.affiliations.len(), 2);
        assert_eq!(data.affiliations[0], (1, "Dept A".to_string()));
        assert_eq!(data.affiliations[1], (2, "Dept B".to_string()));
        assert_eq!(data.authors[1].aff_numbers, vec![2]);
    }

    #[test]
    fn aff_numbers_sorted_ascending() {
        // Author whose institutions resolve to numbers 2 then 1 should display as 1,2
        let rows = vec![
            AuthorRow {
                email: "x@x.com".into(),
                first: "X".into(), middle: "".into(), last: "X".into(), suffix: "".into(),
                institutions: vec!["Dept A".into()],
                is_corresponding: false, is_equal_contribution: false, orcid: "".into(),
            },
            AuthorRow {
                email: "y@y.com".into(),
                first: "Y".into(), middle: "".into(), last: "Y".into(), suffix: "".into(),
                // Lists Dept B first, but Dept B gets number 2 — should still show 1,2
                institutions: vec!["Dept B".into(), "Dept A".into()],
                is_corresponding: false, is_equal_contribution: false, orcid: "".into(),
            },
        ];
        let data = build_author_data(&rows);
        assert_eq!(data.authors[1].aff_numbers, vec![1, 2]);
    }

    #[test]
    fn normalize_aff_collapses_whitespace_and_case() {
        assert_eq!(normalize_aff("  Dept of Genetics,  University of Utah. "), "dept of genetics, university of utah");
        assert_eq!(normalize_aff("University of UTAH"), "university of utah");
        assert_eq!(normalize_aff("MIT."), "mit");
    }

    #[test]
    fn find_merge_suggestions_exact_dup() {
        let rows = vec![
            AuthorRow {
                email: "".into(), first: "A".into(), middle: "".into(), last: "A".into(),
                suffix: "".into(), institutions: vec!["Univ X".into()],
                is_corresponding: false, is_equal_contribution: false, orcid: "".into(),
            },
            AuthorRow {
                email: "".into(), first: "B".into(), middle: "".into(), last: "B".into(),
                suffix: "".into(), institutions: vec!["Univ X".into()],
                is_corresponding: false, is_equal_contribution: false, orcid: "".into(),
            },
        ];
        // Exact duplicates are already handled by build_author_data; find_merge_suggestions
        // should not re-surface them since they're the same raw string (not seen twice).
        let s = find_merge_suggestions(&rows);
        assert!(s.is_empty(), "exact dupes already deduped upstream: {:?}", s);
    }

    #[test]
    fn find_merge_suggestions_whitespace_variant() {
        let rows = vec![
            AuthorRow {
                email: "".into(), first: "A".into(), middle: "".into(), last: "A".into(),
                suffix: "".into(), institutions: vec!["Univ  X".into()],
                is_corresponding: false, is_equal_contribution: false, orcid: "".into(),
            },
            AuthorRow {
                email: "".into(), first: "B".into(), middle: "".into(), last: "B".into(),
                suffix: "".into(), institutions: vec!["Univ X".into()],
                is_corresponding: false, is_equal_contribution: false, orcid: "".into(),
            },
        ];
        let s = find_merge_suggestions(&rows);
        assert_eq!(s.len(), 1);
        // First occurrence ("Univ  X") is kept, second ("Univ X") dropped
        assert_eq!(s[0].1, "Univ  X", "keep first occurrence");
    }

    #[test]
    fn find_merge_suggestions_substring_containment() {
        let rows = vec![
            AuthorRow {
                email: "".into(), first: "A".into(), middle: "".into(), last: "A".into(),
                suffix: "".into(),
                institutions: vec!["Dept of Human Genetics, University of Utah".into()],
                is_corresponding: false, is_equal_contribution: false, orcid: "".into(),
            },
            AuthorRow {
                email: "".into(), first: "B".into(), middle: "".into(), last: "B".into(),
                suffix: "".into(),
                institutions: vec!["University of Utah".into()],
                is_corresponding: false, is_equal_contribution: false, orcid: "".into(),
            },
        ];
        let s = find_merge_suggestions(&rows);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].0, "University of Utah", "shorter form should be dropped");
        assert_eq!(s[0].1, "Dept of Human Genetics, University of Utah", "longer form kept");
    }

    #[test]
    fn find_merge_suggestions_no_match() {
        let rows = vec![
            AuthorRow {
                email: "".into(), first: "A".into(), middle: "".into(), last: "A".into(),
                suffix: "".into(), institutions: vec!["MIT".into()],
                is_corresponding: false, is_equal_contribution: false, orcid: "".into(),
            },
            AuthorRow {
                email: "".into(), first: "B".into(), middle: "".into(), last: "B".into(),
                suffix: "".into(), institutions: vec!["Stanford University".into()],
                is_corresponding: false, is_equal_contribution: false, orcid: "".into(),
            },
        ];
        assert!(find_merge_suggestions(&rows).is_empty());
    }

    #[test]
    fn author_line_plain_format() {
        let rows = make_rows();
        let data = build_author_data(&rows);
        let opts = FormatOptions::default();
        let line = author_line_plain(&data, &opts);
        assert!(line.starts_with("Alice B. Smith1,2*\u{2020}"), "got: {line}");
        assert!(line.contains("Bob Jones2"));
    }
}
