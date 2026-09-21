use crate::render::document::{DocumentLine, LineKind};

pub fn layout_wrapped_lines(
    lines: &[DocumentLine],
    heading_size: f32,
    body_size: f32,
    max_width_pt: f32,
) -> Vec<DocumentLine> {
    let mut wrapped = Vec::new();
    for line in lines {
        if line.text.is_empty() {
            wrapped.push(line.clone());
            continue;
        }
        let size = match line.kind {
            LineKind::Heading => heading_size,
            LineKind::Body => body_size,
        };
        let bold = matches!(line.kind, LineKind::Heading);
        for segment in wrap_text(&line.text, max_width_pt, size, bold) {
            wrapped.push(DocumentLine {
                kind: line.kind.clone(),
                text: segment,
            });
        }
    }
    wrapped
}

pub fn wrap_text(text: &str, max_width_pt: f32, font_size: f32, bold: bool) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    if estimate_text_width_pt(text, font_size, bold) <= max_width_pt {
        return vec![text.to_string()];
    }

    let indent = if text.starts_with('•') { "  " } else { "" };
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if estimate_text_width_pt(&candidate, font_size, bold) <= max_width_pt {
            current = candidate;
            continue;
        }
        if !current.is_empty() {
            lines.push(current);
        }
        let continuation = if lines.is_empty() {
            word.to_string()
        } else {
            format!("{indent}{word}")
        };
        if estimate_text_width_pt(&continuation, font_size, bold) <= max_width_pt {
            current = continuation;
        } else {
            // Hard-split oversized tokens so layout never overflows the page width.
            let chunks = split_oversized_token(&continuation, max_width_pt, font_size, bold);
            if let Some((last, rest)) = chunks.split_last() {
                lines.extend(rest.iter().cloned());
                current = last.clone();
            } else {
                current = continuation;
            }
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        vec![text.to_string()]
    } else {
        lines
    }
}

pub(crate) fn split_oversized_token(
    token: &str,
    max_width_pt: f32,
    font_size: f32,
    bold: bool,
) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    for character in token.chars() {
        let candidate = format!("{current}{character}");
        if !current.is_empty() && estimate_text_width_pt(&candidate, font_size, bold) > max_width_pt
        {
            chunks.push(current);
            current = character.to_string();
        } else {
            current = candidate;
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

pub(crate) fn estimate_text_width_pt(text: &str, font_size: f32, bold: bool) -> f32 {
    let average = if bold { 0.55 } else { 0.50 };
    text.chars()
        .map(|character| {
            let factor = match character {
                'i' | 'l' | 'I' | 'j' | 't' | 'f' | 'r' | '.' | ',' | ':' | ';' | '!' | '|'
                | '\'' | '•' => 0.28,
                'm' | 'w' | 'M' | 'W' => 0.85,
                ' ' => 0.28,
                _ => average,
            };
            factor * font_size
        })
        .sum()
}
