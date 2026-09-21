use crate::model::{Resume, SectionId, Theme};
use crate::render::locale::localized_label;
use crate::util::{format_date_range, join_non_empty};

#[derive(Debug, Clone)]
pub enum LineKind {
    Heading,
    Body,
}

#[derive(Debug, Clone)]
pub struct DocumentLine {
    pub kind: LineKind,
    pub text: String,
}

pub fn render_lines(resume: &Resume, theme: &Theme) -> Vec<DocumentLine> {
    let locale = resume.locale.as_deref();
    let mut lines = Vec::new();
    for section in theme.visible_sections() {
        match section {
            SectionId::Basics => {
                lines.push(DocumentLine {
                    kind: LineKind::Heading,
                    text: resume.basics.name.clone(),
                });
                if let Some(headline) = &resume.basics.headline {
                    lines.push(DocumentLine {
                        kind: LineKind::Body,
                        text: headline.clone(),
                    });
                }
                lines.push(DocumentLine {
                    kind: LineKind::Body,
                    text: join_non_empty([
                        Some(resume.basics.email.as_str()),
                        resume.basics.phone.as_deref(),
                        resume.basics.location.as_deref(),
                    ]),
                });
                if !resume.basics.links.is_empty() {
                    lines.push(DocumentLine {
                        kind: LineKind::Body,
                        text: resume
                            .basics
                            .links
                            .iter()
                            .map(|link| format!("{}: {}", link.label, link.url))
                            .collect::<Vec<_>>()
                            .join(" | "),
                    });
                }
                lines.push(blank_line());
            }
            SectionId::Summary => {
                if let Some(summary) = &resume.basics.summary {
                    lines.push(section_heading(localized_label("summary", locale)));
                    lines.push(DocumentLine {
                        kind: LineKind::Body,
                        text: summary.clone(),
                    });
                    lines.push(blank_line());
                }
            }
            SectionId::Work => {
                if !resume.work.is_empty() {
                    lines.push(section_heading(localized_label("work", locale)));
                    for entry in &resume.work {
                        lines.push(DocumentLine {
                            kind: LineKind::Body,
                            text: format!(
                                "{} — {} ({})",
                                entry.position,
                                entry.company,
                                join_non_empty([
                                    format_date_range(
                                        entry.start_date.as_deref(),
                                        entry.end_date.as_deref()
                                    )
                                    .as_deref(),
                                    entry.location.as_deref(),
                                    None,
                                ])
                            ),
                        });
                        if let Some(summary) = &entry.summary {
                            lines.push(DocumentLine {
                                kind: LineKind::Body,
                                text: summary.clone(),
                            });
                        }
                        for bullet in &entry.highlights {
                            lines.push(DocumentLine {
                                kind: LineKind::Body,
                                text: format!("• {bullet}"),
                            });
                        }
                        lines.push(blank_line());
                    }
                }
            }
            SectionId::Education => {
                if !resume.education.is_empty() {
                    lines.push(section_heading(localized_label("education", locale)));
                    for entry in &resume.education {
                        lines.push(DocumentLine {
                            kind: LineKind::Body,
                            text: format!(
                                "{}{} — {}{}",
                                entry.study_type.clone().unwrap_or_default(),
                                if entry.study_type.is_some() { " " } else { "" },
                                entry.area,
                                if entry.institution.is_empty() {
                                    "".to_string()
                                } else {
                                    format!(" | {}", entry.institution)
                                }
                            ),
                        });
                        let details = join_non_empty([
                            format_date_range(
                                entry.start_date.as_deref(),
                                entry.end_date.as_deref(),
                            )
                            .as_deref(),
                            entry.location.as_deref(),
                            entry.score.as_deref(),
                        ]);
                        if !details.is_empty() {
                            lines.push(DocumentLine {
                                kind: LineKind::Body,
                                text: details,
                            });
                        }
                        lines.push(blank_line());
                    }
                }
            }
            SectionId::Skills => {
                if !resume.skills.is_empty() {
                    lines.push(section_heading(localized_label("skills", locale)));
                    for entry in &resume.skills {
                        lines.push(DocumentLine {
                            kind: LineKind::Body,
                            text: format!("{}: {}", entry.category, entry.items.join(", ")),
                        });
                    }
                    lines.push(blank_line());
                }
            }
            SectionId::Certifications => {
                if !resume.certifications.is_empty() {
                    lines.push(section_heading(localized_label("certifications", locale)));
                    for entry in &resume.certifications {
                        lines.push(DocumentLine {
                            kind: LineKind::Body,
                            text: join_non_empty([
                                Some(entry.name.as_str()),
                                entry.issuer.as_deref(),
                                entry.date.as_deref(),
                            ]),
                        });
                        if let Some(summary) = &entry.summary {
                            lines.push(DocumentLine {
                                kind: LineKind::Body,
                                text: summary.clone(),
                            });
                        }
                    }
                    lines.push(blank_line());
                }
            }
            SectionId::Projects => {
                if !resume.projects.is_empty() {
                    lines.push(section_heading(localized_label("projects", locale)));
                    for entry in &resume.projects {
                        lines.push(DocumentLine {
                            kind: LineKind::Body,
                            text: join_non_empty([
                                Some(entry.name.as_str()),
                                entry.url.as_deref(),
                                None,
                            ]),
                        });
                        if let Some(summary) = &entry.summary {
                            lines.push(DocumentLine {
                                kind: LineKind::Body,
                                text: summary.clone(),
                            });
                        }
                        for bullet in &entry.highlights {
                            lines.push(DocumentLine {
                                kind: LineKind::Body,
                                text: format!("• {bullet}"),
                            });
                        }
                        lines.push(blank_line());
                    }
                }
            }
            SectionId::Languages => {
                if !resume.languages.is_empty() {
                    lines.push(section_heading(localized_label("languages", locale)));
                    for entry in &resume.languages {
                        lines.push(DocumentLine {
                            kind: LineKind::Body,
                            text: join_non_empty([
                                Some(entry.name.as_str()),
                                entry.fluency.as_deref(),
                                None,
                            ]),
                        });
                    }
                    lines.push(blank_line());
                }
            }
            SectionId::Awards => {
                if !resume.awards.is_empty() {
                    lines.push(section_heading(localized_label("awards", locale)));
                    for entry in &resume.awards {
                        lines.push(DocumentLine {
                            kind: LineKind::Body,
                            text: join_non_empty([
                                Some(entry.title.as_str()),
                                entry.issuer.as_deref(),
                                entry.date.as_deref(),
                            ]),
                        });
                        if let Some(summary) = &entry.summary {
                            lines.push(DocumentLine {
                                kind: LineKind::Body,
                                text: summary.clone(),
                            });
                        }
                    }
                    lines.push(blank_line());
                }
            }
        }
    }
    while matches!(lines.last(), Some(DocumentLine { text, .. }) if text.is_empty()) {
        lines.pop();
    }
    lines
}

pub(crate) fn blank_line() -> DocumentLine {
    DocumentLine {
        kind: LineKind::Body,
        text: String::new(),
    }
}

pub(crate) fn section_heading(title: &str) -> DocumentLine {
    DocumentLine {
        kind: LineKind::Heading,
        text: title.to_string(),
    }
}
