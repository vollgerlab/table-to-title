use anyhow::{anyhow, Result};
#[cfg(feature = "server")]
use std::io::Cursor;

#[derive(Debug, Clone)]
pub struct AuthorRow {
    #[allow(dead_code)]
    pub email: String,
    pub first: String,
    pub middle: String,
    pub last: String,
    pub suffix: String,
    pub institutions: Vec<String>,
    pub is_corresponding: bool,
    pub is_equal_contribution: bool,
    pub orcid: String,
}

/// Parse biorxiv-format TSV text into author rows.
/// Handles UTF-8 BOM and missing optional columns.
pub fn parse_tsv(text: &str) -> Result<Vec<AuthorRow>> {
    // Strip BOM if present
    let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .from_reader(text.as_bytes());

    // Build column index from headers
    let headers = rdr.headers()?.clone();
    let col = |name: &str| -> Option<usize> {
        headers.iter().position(|h| h.trim() == name)
    };

    let idx_email   = col("Email");
    let idx_first   = col("First Name").ok_or_else(|| anyhow!("Missing column: First Name"))?;
    let idx_middle  = col("Middle Name(s)/Initial(s)");
    let idx_last    = col("Last Name").ok_or_else(|| anyhow!("Missing column: Last Name"))?;
    let idx_suffix  = col("Suffix");
    let idx_inst    = col("Institution").ok_or_else(|| anyhow!("Missing column: Institution"))?;
    let idx_corresp = col("Corresponding Author");
    let idx_equal   = col("Equal Contribution");
    let idx_orcid   = col("ORCiD");

    let get = |record: &csv::StringRecord, idx: usize| -> String {
        record.get(idx).unwrap_or("").trim().to_string()
    };
    let get_opt = |record: &csv::StringRecord, idx: Option<usize>| -> String {
        idx.map(|i| record.get(i).unwrap_or("").trim().to_string())
            .unwrap_or_default()
    };

    let mut rows = Vec::new();
    for result in rdr.records() {
        let record = result?;

        let first = get(&record, idx_first);
        let last  = get(&record, idx_last);

        // Skip rows with no name
        if first.is_empty() && last.is_empty() {
            continue;
        }

        let inst_str = get(&record, idx_inst);
        let institutions: Vec<String> = inst_str
            .split(';')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let corresp = get_opt(&record, idx_corresp);
        let is_corresponding = corresp.to_lowercase() == "yes";
        let equal = get_opt(&record, idx_equal);
        let is_equal_contribution = equal.to_lowercase() == "yes";

        rows.push(AuthorRow {
            email:                get_opt(&record, idx_email),
            first,
            middle:               get_opt(&record, idx_middle),
            last,
            suffix:               get_opt(&record, idx_suffix),
            institutions,
            is_corresponding,
            is_equal_contribution,
            orcid:                get_opt(&record, idx_orcid),
        });
    }

    Ok(rows)
}

#[cfg(feature = "server")]
/// Convert Excel file bytes (.xlsx or .xls) to TSV text.
/// Reads the first worksheet and serialises all cells as tab-separated values.
pub fn excel_bytes_to_tsv(bytes: &[u8]) -> Result<String> {
    use calamine::{Reader, Xlsx, Xls, open_workbook_from_rs};

    // Detect format from magic bytes
    let is_xlsx = bytes.starts_with(&[0x50, 0x4B]); // ZIP / Office Open XML
    let is_xls  = bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]); // OLE2

    if !is_xlsx && !is_xls {
        anyhow::bail!("Unrecognised file format — expected .xlsx or .xls");
    }

    let rows: Vec<Vec<String>> = if is_xlsx {
        let cursor = Cursor::new(bytes);
        let mut wb: Xlsx<_> = open_workbook_from_rs(cursor)?;
        let sheet = wb
            .worksheet_range_at(0)
            .ok_or_else(|| anyhow!("Excel file has no sheets"))??;
        sheet
            .rows()
            .map(|row| row.iter().map(|cell| cell_to_string(cell)).collect())
            .collect()
    } else {
        let cursor = Cursor::new(bytes);
        let mut wb: Xls<_> = open_workbook_from_rs(cursor)?;
        let sheet = wb
            .worksheet_range_at(0)
            .ok_or_else(|| anyhow!("Excel file has no sheets"))??;
        sheet
            .rows()
            .map(|row| row.iter().map(|cell| cell_to_string(cell)).collect())
            .collect()
    };

    let tsv = rows
        .iter()
        .map(|row| row.join("\t"))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(tsv)
}

#[cfg(feature = "server")]
fn cell_to_string(cell: &calamine::Data) -> String {
    use calamine::Data;
    match cell {
        Data::Empty              => String::new(),
        Data::String(s)          => s.trim().to_string(),
        Data::Float(f)           => f.to_string(),
        Data::Int(i)             => i.to_string(),
        Data::Bool(b)            => b.to_string(),
        Data::Error(e)           => format!("{:?}", e),
        Data::DateTime(dt)       => dt.to_string(),
        Data::DateTimeIso(s)     => s.clone(),
        Data::DurationIso(s)     => s.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "Email\tFirst Name\tMiddle Name(s)/Initial(s)\tLast Name\tSuffix\tInstitution\tCorresponding Author\tHome Page URL\tCollaborative Group/Consortium\tORCiD
a@example.com\tAlice\tB.\tSmith\t\tDept A, Univ X\tyes\t\t\t0000-0001-2345-6789
b@example.com\tBob\t\tJones\t\tDept B, Univ Y; Dept A, Univ X\t\t\t\t";

    #[test]
    fn parses_two_authors() {
        let rows = parse_tsv(SAMPLE).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].first, "Alice");
        assert_eq!(rows[0].middle, "B.");
        assert!(rows[0].is_corresponding);
        assert_eq!(rows[0].institutions, vec!["Dept A, Univ X"]);
        assert_eq!(rows[1].first, "Bob");
        assert!(!rows[1].is_corresponding);
        assert_eq!(rows[1].institutions.len(), 2);
    }

    #[test]
    fn strips_bom() {
        let with_bom = format!("\u{FEFF}{}", SAMPLE);
        let rows = parse_tsv(&with_bom).unwrap();
        assert_eq!(rows.len(), 2);
    }
}
