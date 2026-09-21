pub fn is_portuguese_locale(locale: Option<&str>) -> bool {
    locale
        .map(|value| {
            let normalized = value.to_ascii_lowercase();
            normalized == "pt" || normalized.starts_with("pt-") || normalized.starts_with("pt_")
        })
        .unwrap_or(false)
}

pub fn localized_label(key: &str, locale: Option<&str>) -> &'static str {
    let portuguese = is_portuguese_locale(locale);
    match (key, portuguese) {
        ("summary", true) => "Resumo",
        ("summary", false) => "Summary",
        ("work", true) => "Experiência Profissional",
        ("work", false) => "Work Experience",
        ("education", true) => "Educação",
        ("education", false) => "Education",
        ("skills", true) => "Habilidades",
        ("skills", false) => "Skills",
        ("certifications", true) => "Certificações",
        ("certifications", false) => "Certifications",
        ("projects", true) => "Projetos",
        ("projects", false) => "Projects",
        ("languages", true) => "Idiomas",
        ("languages", false) => "Languages",
        ("awards", true) => "Realizações",
        ("awards", false) => "Awards",
        _ => "Section",
    }
}
