pub mod export;
pub mod format;
pub mod parse;

// ── WASM exports ──────────────────────────────────────────────────────────────
// Only compiled when building with --features wasm (i.e. wasm-pack build)

#[cfg(feature = "wasm")]
mod wasm_api {
    use wasm_bindgen::prelude::*;

    use crate::{export, format, parse};

    fn opts(show_corresponding: bool, show_orcid: bool, show_equal: bool) -> format::FormatOptions {
        format::FormatOptions { show_corresponding, show_orcid, show_equal }
    }

    /// Parse TSV and return a complete preview HTML fragment.
    #[wasm_bindgen]
    pub fn preview(
        tsv: &str,
        title: &str,
        show_corresponding: bool,
        show_orcid: bool,
        show_equal: bool,
    ) -> Result<String, JsValue> {
        let rows = parse::parse_tsv(tsv).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let data = format::build_author_data(&rows);
        let opts = opts(show_corresponding, show_orcid, show_equal);
        Ok(format::build_preview_html(&data, &opts, title))
    }

    /// Parse TSV and return a .docx file as raw bytes (becomes Uint8Array in JS).
    #[wasm_bindgen]
    pub fn generate_docx(
        tsv: &str,
        title: &str,
        show_corresponding: bool,
        show_orcid: bool,
        show_equal: bool,
    ) -> Result<Vec<u8>, JsValue> {
        let rows = parse::parse_tsv(tsv).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let data = format::build_author_data(&rows);
        let opts = opts(show_corresponding, show_orcid, show_equal);
        export::build_docx(&data, &opts, title).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Analyse a TSV and return a JSON array of merge suggestions.
    /// Each element: `{"drop": "...", "keep": "...", "reason": "..."}`.
    #[wasm_bindgen]
    pub fn suggest_merges(tsv: &str) -> Result<String, JsValue> {
        let rows = parse::parse_tsv(tsv).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let pairs = format::find_merge_suggestions(&rows);
        let json: Vec<serde_json::Value> = pairs
            .into_iter()
            .map(|(drop, keep)| {
                let reason = if format::normalize_aff(&drop) == format::normalize_aff(&keep) {
                    "whitespace/case variant"
                } else {
                    "less specific (substring)"
                };
                serde_json::json!({ "drop": drop, "keep": keep, "reason": reason })
            })
            .collect();
        serde_json::to_string(&json).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Apply a merge map to the Institution column of a TSV and return the cleaned TSV.
    /// `merges_json` is a JSON object mapping drop-string → keep-string.
    #[wasm_bindgen]
    pub fn apply_merges(tsv: &str, merges_json: &str) -> Result<String, JsValue> {
        use std::collections::HashMap;
        let merges: HashMap<String, String> = serde_json::from_str(merges_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid merges JSON: {e}")))?;
        if merges.is_empty() {
            return Ok(tsv.to_string());
        }

        let text = tsv.strip_prefix('\u{FEFF}').unwrap_or(tsv);
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return Ok(tsv.to_string());
        }

        // Locate the Institution column index from the header row.
        let headers: Vec<&str> = lines[0].split('\t').collect();
        let inst_col = match headers.iter().position(|h| h.trim() == "Institution") {
            Some(i) => i,
            None => return Ok(tsv.to_string()),
        };

        let mut out: Vec<String> = Vec::with_capacity(lines.len());
        out.push(lines[0].to_string());

        for line in &lines[1..] {
            if line.trim().is_empty() {
                out.push(line.to_string());
                continue;
            }
            let mut fields: Vec<String> = line.split('\t').map(str::to_string).collect();
            if let Some(cell) = fields.get_mut(inst_col) {
                let mut institutions: Vec<String> = cell
                    .split(';')
                    .map(|s| {
                        let t = s.trim().to_string();
                        merges.get(&t).cloned().unwrap_or(t)
                    })
                    .collect();
                // Deduplicate in case two institutions merged to the same string.
                let mut seen_insts: Vec<String> = Vec::new();
                for inst in institutions.drain(..) {
                    if !seen_insts.contains(&inst) {
                        seen_insts.push(inst);
                    }
                }
                *cell = seen_insts.join("; ");
            }
            out.push(fields.join("\t"));
        }

        Ok(out.join("\n"))
    }

    /// Parse TSV and return a plain-text string.
    #[wasm_bindgen]
    pub fn generate_txt(
        tsv: &str,
        title: &str,
        show_corresponding: bool,
        show_orcid: bool,
        show_equal: bool,
    ) -> Result<String, JsValue> {
        let rows = parse::parse_tsv(tsv).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let data = format::build_author_data(&rows);
        let opts = opts(show_corresponding, show_orcid, show_equal);
        Ok(export::build_txt(&data, &opts, title))
    }
}
