use std::fs;

use printpdf::{
    BuiltinFont, Color, Mm, Op, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Pt, Rgb,
    TextItem, TextMatrix,
};

use crate::build::BuildOptions;
use crate::constants::DEFAULT_PAGE_MARGIN_PT;
use crate::error::AppError;
use crate::model::{GeneratedOutput, OutputFormat, Resume, Theme};
use crate::output::write::ensure_can_write;
use crate::render::document::{DocumentLine, LineKind};
use crate::render::wrap::layout_wrapped_lines;
use crate::util::{hex_to_rgb, pt_to_mm};

pub fn write_pdf(
    resume: &Resume,
    theme: &Theme,
    lines: &[DocumentLine],
    options: &BuildOptions,
) -> Result<GeneratedOutput, AppError> {
    let destination = options
        .output_dir
        .join(format!("{}.pdf", options.output_name));
    ensure_can_write(&destination, options)?;
    let mut doc = PdfDocument::new(&resume.basics.name);
    let mut pages = Vec::new();
    let heading_size = theme.typography.heading_size_pt;
    let body_size = theme.typography.base_size_pt;
    let margin_mm = pt_to_mm(theme.spacing.page_margin_pt.max(DEFAULT_PAGE_MARGIN_PT));
    let width_mm = 210.0f32;
    let height_mm = 297.0f32;
    let max_width_pt = Mm(width_mm - (margin_mm * 2.0)).into_pt().0;
    let layout_lines = layout_wrapped_lines(lines, heading_size, body_size, max_width_pt);
    let mut y = height_mm - margin_mm;
    let mut ops = start_text_ops();

    for line in &layout_lines {
        let size = match line.kind {
            LineKind::Heading => heading_size,
            LineKind::Body => body_size,
        };
        let line_height_pt = if line.text.is_empty() {
            theme.spacing.section_gap_pt.max(theme.spacing.item_gap_pt)
        } else {
            (size * 1.15) + theme.spacing.item_gap_pt
        };
        let consumed = pt_to_mm(line_height_pt);
        if y - consumed < margin_mm {
            ops.push(Op::EndTextSection);
            ops.push(Op::RestoreGraphicsState);
            pages.push(PdfPage::new(Mm(width_mm), Mm(height_mm), ops));
            ops = start_text_ops();
            y = height_mm - margin_mm;
        }
        if !line.text.is_empty() {
            ops.push(Op::SetFillColor {
                col: color_for_line(theme, &line.kind),
            });
            ops.push(Op::SetFont {
                font: font_for_line(&line.kind),
                size: Pt(size),
            });
            // printpdf serializes SetTextCursor as relative Td; use Tm for absolute page coords.
            ops.push(Op::SetTextMatrix {
                matrix: TextMatrix::Translate(Mm(margin_mm).into(), Mm(y).into()),
            });
            ops.push(Op::ShowText {
                items: vec![TextItem::Text(line.text.clone())],
            });
        }
        y -= consumed;
    }

    ops.push(Op::EndTextSection);
    ops.push(Op::RestoreGraphicsState);
    pages.push(PdfPage::new(Mm(width_mm), Mm(height_mm), ops));

    let document = doc.with_pages(pages);
    let bytes = document.save(&PdfSaveOptions::default(), &mut Vec::new());
    fs::write(&destination, &bytes).map_err(|error| AppError::FileWrite {
        path: destination.display().to_string(),
        message: error.to_string(),
    })?;
    Ok(GeneratedOutput {
        format: OutputFormat::Pdf,
        path: destination,
        bytes: bytes.len() as u64,
    })
}

fn start_text_ops() -> Vec<Op> {
    vec![Op::SaveGraphicsState, Op::StartTextSection]
}

fn color_for_line(theme: &Theme, kind: &LineKind) -> Color {
    let hex = match kind {
        LineKind::Heading => &theme.palette.primary,
        LineKind::Body => &theme.palette.text,
    };
    let (r, g, b) = hex_to_rgb(hex);
    Color::Rgb(Rgb::new(r, g, b, None))
}

fn font_for_line(kind: &LineKind) -> PdfFontHandle {
    match kind {
        LineKind::Heading => PdfFontHandle::Builtin(BuiltinFont::HelveticaBold),
        LineKind::Body => PdfFontHandle::Builtin(BuiltinFont::Helvetica),
    }
}
