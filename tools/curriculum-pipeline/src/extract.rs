use std::path::Path;

pub fn extract_text_from_file(pdf_path: &Path) -> anyhow::Result<String> {
    let result = pdf_inspector::process_pdf(pdf_path)
        .map_err(|e| anyhow::anyhow!("PDF processing failed: {e}"))?;
    result.markdown.ok_or_else(|| {
        anyhow::anyhow!(
            "PDF type {:?} produced no markdown output (scanned/image PDF?)",
            result.pdf_type
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use printpdf::*;
    use std::io::{BufWriter, Cursor};

    #[test]
    fn extracts_known_text_from_a_generated_pdf() {
        let (doc, page1, layer1) = PdfDocument::new("fixture", Mm(210.0), Mm(297.0), "Layer 1");
        let layer = doc.get_page(page1).get_layer(layer1);
        let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
        layer.use_text(
            "Photosynthesis converts light energy into chemical energy",
            24.0,
            Mm(10.0),
            Mm(280.0),
            &font,
        );
        let mut buf = Vec::new();
        doc.save(&mut BufWriter::new(Cursor::new(&mut buf)))
            .unwrap();

        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), &buf).unwrap();
        let text = extract_text_from_file(tmp.path()).unwrap();
        assert!(
            text.contains("Photosynthesis"),
            "expected extracted text to contain the fixture's text, got: {text:?}"
        );
    }
}
