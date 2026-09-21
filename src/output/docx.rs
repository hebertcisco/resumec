use std::fs;

use docx_rs::{Docx, Paragraph, Run};

use crate::build::BuildOptions;
use crate::error::AppError;
use crate::model::{GeneratedOutput, OutputFormat, Resume, Theme};
use crate::output::write::ensure_can_write;
use crate::render::document::{DocumentLine, LineKind};
use crate::util::strip_hash;

pub fn write_docx(
    _resume: &Resume,
    theme: &Theme,
    lines: &[DocumentLine],
    options: &BuildOptions,
) -> Result<GeneratedOutput, AppError> {
    let destination = options
        .output_dir
        .join(format!("{}.docx", options.output_name));
    ensure_can_write(&destination, options)?;
    let file = fs::File::create(&destination).map_err(|error| AppError::FileWrite {
        path: destination.display().to_string(),
        message: error.to_string(),
    })?;
    let mut docx = Docx::new();
    for line in lines {
        let mut run = Run::new().add_text(line.text.clone());
        match line.kind {
            LineKind::Heading => {
                run = run
                    .bold()
                    .size((theme.typography.heading_size_pt * 2.0) as usize)
                    .color(strip_hash(&theme.palette.primary));
            }
            LineKind::Body => {
                run = run
                    .size((theme.typography.base_size_pt * 2.0) as usize)
                    .color(strip_hash(&theme.palette.text));
            }
        }
        docx = docx.add_paragraph(Paragraph::new().add_run(run));
    }
    docx.build()
        .pack(file)
        .map_err(|error| AppError::FileWrite {
            path: destination.display().to_string(),
            message: error.to_string(),
        })?;
    let bytes = fs::metadata(&destination)
        .map_err(|error| AppError::FileRead {
            path: destination.display().to_string(),
            message: error.to_string(),
        })?
        .len();
    Ok(GeneratedOutput {
        format: OutputFormat::Docx,
        path: destination,
        bytes,
    })
}
