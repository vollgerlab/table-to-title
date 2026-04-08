use std::io::Cursor;

use anyhow::Result;
use docx_rs::{AlignmentType, Docx, Paragraph, Run, VertAlignType};

use crate::format::{AuthorData, FormatOptions, author_line_plain, legend_lines};

/// Build a .docx file with superscript affiliation numbers and symbols.
/// Returns raw bytes suitable for sending as an HTTP response.
pub fn build_docx(data: &AuthorData, opts: &FormatOptions, title: &str) -> Result<Vec<u8>> {
    let mut doc = Docx::new();

    // Title paragraph — bold, centered
    if !title.trim().is_empty() {
        let mut title_run = Run::new().add_text(title.trim());
        title_run.run_property = title_run.run_property.bold();
        doc = doc.add_paragraph(
            Paragraph::new()
                .align(AlignmentType::Center)
                .add_run(title_run),
        );
        doc = doc.add_paragraph(Paragraph::new());
    }

    // Author line paragraph
    let mut para = Paragraph::new();
    for (i, author) in data.authors.iter().enumerate() {
        if i > 0 {
            para = para.add_run(Run::new().add_text(", "));
        }

        para = para.add_run(Run::new().add_text(&author.name));

        // Build superscript: [*][†][1,2,3]
        let mut sup = String::new();
        if opts.show_equal && author.is_equal {
            sup.push('*');
        }
        if opts.show_corresponding && author.is_corresponding {
            sup.push('\u{2020}');
        }
        if !author.aff_numbers.is_empty() {
            sup.push_str(
                &author.aff_numbers.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(","),
            );
        }
        if !sup.is_empty() {
            let mut run = Run::new().add_text(&sup);
            run.run_property = run.run_property.vert_align(VertAlignType::SuperScript);
            para = para.add_run(run);
        }
    }
    doc = doc.add_paragraph(para);
    doc = doc.add_paragraph(Paragraph::new());

    // Numbered affiliation list
    for (num, aff) in &data.affiliations {
        doc = doc.add_paragraph(
            Paragraph::new().add_run(Run::new().add_text(&format!("{}. {}", num, aff))),
        );
    }

    // Legend lines (* equal contribution, † corresponding)
    let legends = legend_lines(data, opts);
    if !legends.is_empty() {
        doc = doc.add_paragraph(Paragraph::new());
        for legend in &legends {
            // First char(s) may be a symbol — render as superscript, rest at baseline
            let symbol_end = legend.chars().next().map(|c| c.len_utf8()).unwrap_or(0);
            let symbol = &legend[..symbol_end];
            let rest   = &legend[symbol_end..];
            let mut legend_para = Paragraph::new();
            let mut sym_run = Run::new().add_text(symbol);
            sym_run.run_property = sym_run.run_property.vert_align(VertAlignType::SuperScript);
            legend_para = legend_para.add_run(sym_run).add_run(Run::new().add_text(rest));
            doc = doc.add_paragraph(legend_para);
        }
    }

    // Optional ORCiD section
    if opts.show_orcid {
        let orcids: Vec<_> = data
            .authors
            .iter()
            .filter_map(|a| a.orcid.as_ref().map(|o| (a.name.as_str(), o.as_str())))
            .collect();
        if !orcids.is_empty() {
            doc = doc.add_paragraph(Paragraph::new());
            doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text("ORCiD:")));
            for (name, orcid) in orcids {
                doc = doc.add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_text(&format!("{}: {}", name, orcid))),
                );
            }
        }
    }

    let mut cursor = Cursor::new(Vec::<u8>::new());
    doc.build().pack(&mut cursor)?;
    Ok(cursor.into_inner())
}

/// Build plain text output.
pub fn build_txt(data: &AuthorData, opts: &FormatOptions, title: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    if !title.trim().is_empty() {
        lines.push(title.trim().to_string());
        lines.push(String::new());
    }
    lines.push(author_line_plain(data, opts));
    lines.push(String::new());

    for (num, aff) in &data.affiliations {
        lines.push(format!("{}. {}", num, aff));
    }

    let legends = legend_lines(data, opts);
    if !legends.is_empty() {
        lines.push(String::new());
        lines.extend(legends);
    }

    if opts.show_orcid {
        let orcids: Vec<_> = data
            .authors
            .iter()
            .filter_map(|a| a.orcid.as_ref().map(|o| (a.name.as_str(), o.as_str())))
            .collect();
        if !orcids.is_empty() {
            lines.push(String::new());
            lines.push("ORCiD:".to_string());
            for (name, orcid) in orcids {
                lines.push(format!("{}: {}", name, orcid));
            }
        }
    }

    lines.join("\n")
}
