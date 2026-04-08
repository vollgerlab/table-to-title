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
