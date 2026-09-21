use url::Url;

use crate::constants::DEFAULT_PAGE_MARGIN_PT;

pub fn derive_output_name_from_path(path: &std::path::Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(sanitize_filename)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "resume".to_string())
}

pub fn sanitize_filename(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' => character.to_ascii_lowercase(),
            _ => '-',
        })
        .collect::<String>()
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Returns whether a value is a portable, single-component file stem.
///
/// Keeping generated output and theme names to this conservative character set
/// prevents path traversal and avoids platform-specific filename surprises.
pub fn is_safe_file_stem(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

pub fn looks_like_url(value: &str) -> bool {
    matches!(Url::parse(value), Ok(url) if matches!(url.scheme(), "http" | "https"))
}

pub fn join_non_empty<'a, I>(items: I) -> String
where
    I: IntoIterator<Item = Option<&'a str>>,
{
    items
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
        .join(" | ")
}

pub fn format_date_range(start: Option<&str>, end: Option<&str>) -> Option<String> {
    let value = join_non_empty([start, end]);
    if value.is_empty() {
        None
    } else {
        Some(value.replace(" | ", " – "))
    }
}

pub fn strip_hash(color: &str) -> &str {
    color.strip_prefix('#').unwrap_or(color)
}

pub fn hex_to_rgb(color: &str) -> (f32, f32, f32) {
    let stripped = strip_hash(color);
    let parse = |range: std::ops::Range<usize>| {
        u8::from_str_radix(&stripped[range], 16).unwrap_or_default() as f32 / 255.0
    };
    (parse(0..2), parse(2..4), parse(4..6))
}

pub fn is_hex_color(color: &str) -> bool {
    let stripped = strip_hash(color);
    stripped.len() == 6
        && stripped
            .chars()
            .all(|character| character.is_ascii_hexdigit())
}

pub fn pt_to_mm(pt: f32) -> f32 {
    pt * 0.352_778
}

pub fn default_true() -> bool {
    true
}

pub fn default_page_margin_pt() -> f32 {
    DEFAULT_PAGE_MARGIN_PT
}
