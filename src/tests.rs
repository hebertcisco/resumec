use crate::build::{BuildOptions, build_resume_with_paths};
use crate::constants::EXAMPLE_RESUME_YAML;
use crate::model::{OutputFormat, Resume};
use crate::paths::AppPaths;
use crate::render::{localized_label, render_lines, wrap_text};
use crate::resume_io::load_resume;
use crate::theme_io::load_theme_by_name;
use crate::theme_io::parse_theme_content;
use crate::util::is_safe_file_stem;
use std::fs;
use std::path::PathBuf;

use tempfile::tempdir;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn parses_yaml_resume() {
    let path = repo_root().join("examples/resume.yaml");
    let resume = load_resume(&path).expect("yaml should parse");
    assert_eq!(resume.basics.name, "Ana Souza");
}

#[test]
fn validates_resume_schema_version() {
    let mut resume: Resume =
        serde_yaml::from_str(EXAMPLE_RESUME_YAML).expect("fixture should parse");
    resume.schema_version = 2;
    let issues = resume.validate();
    assert!(issues.iter().any(|issue| issue.field == "schema_version"));
}

#[test]
fn accepts_basics_without_contact_fields() {
    let content = EXAMPLE_RESUME_YAML
        .lines()
        .filter(|line| {
            !line.starts_with("  email:")
                && !line.starts_with("  phone:")
                && !line.starts_with("  location:")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let resume: Resume = serde_yaml::from_str(&content).expect("contact fields should be optional");

    assert_eq!(resume.basics.email, None);
    assert_eq!(resume.basics.phone, None);
    assert_eq!(resume.basics.location, None);
    assert!(resume.validate().is_empty());

    let temp = tempdir().expect("temp dir");
    let paths = AppPaths::from_config_dir(temp.path());
    let theme = load_theme_by_name("classic", &paths).expect("classic theme should exist");
    let lines = render_lines(&resume, &theme);
    assert!(
        !lines
            .windows(2)
            .any(|pair| pair.iter().all(|line| line.text.is_empty()))
    );
}

#[test]
fn loads_builtin_theme() {
    let temp = tempdir().expect("temp dir");
    let paths = AppPaths::from_config_dir(temp.path());
    let theme = load_theme_by_name("classic", &paths).expect("classic theme should exist");
    assert_eq!(theme.name, "classic");
}

#[test]
fn builds_both_formats_for_all_builtin_themes() {
    let input = repo_root().join("examples/resume.yaml");
    for theme_name in ["classic", "modern", "minimal"] {
        let temp = tempdir().expect("temp dir");
        let paths = AppPaths::from_config_dir(temp.path().join("config"));
        let result = build_resume_with_paths(
            &input,
            BuildOptions {
                format: OutputFormat::Both,
                theme_name: theme_name.to_string(),
                output_dir: temp.path().to_path_buf(),
                output_name: theme_name.to_string(),
                overwrite: true,
                non_interactive: true,
                json_output: true,
            },
            &paths,
        )
        .expect("build should succeed");
        assert_eq!(result.outputs.len(), 2);
        for output in result.outputs {
            assert!(
                output.path.exists(),
                "{} should exist",
                output.path.display()
            );
            assert!(
                output.bytes > 0,
                "{} should not be empty",
                output.path.display()
            );
        }
    }
}

#[test]
fn pdf_keeps_multiline_resume_content() {
    let input = repo_root().join("examples/resume.yaml");
    let temp = tempdir().expect("temp dir");
    let paths = AppPaths::from_config_dir(temp.path().join("config"));
    let result = build_resume_with_paths(
        &input,
        BuildOptions {
            format: OutputFormat::Pdf,
            theme_name: "modern".to_string(),
            output_dir: temp.path().to_path_buf(),
            output_name: "matrix-check".to_string(),
            overwrite: true,
            non_interactive: true,
            json_output: true,
        },
        &paths,
    )
    .expect("build should succeed");
    let bytes = fs::read(&result.outputs[0].path).expect("pdf should be readable");
    assert!(bytes.starts_with(b"%PDF"));
    assert!(bytes.len() > 1_000);
    assert_eq!(result.outputs[0].bytes, bytes.len() as u64);
}

#[test]
fn wraps_long_lines_and_localizes_portuguese_labels() {
    assert_eq!(
        localized_label("work", Some("pt-BR")),
        "Experiência Profissional"
    );
    assert_eq!(localized_label("work", Some("en-US")), "Work Experience");
    let wrapped = wrap_text(
        "• Desenvolvimento full stack end-to-end com React Native, Laravel e Node.js para produtos white-label em produção.",
        220.0,
        10.5,
        false,
    );
    assert!(wrapped.len() > 1);
    assert!(wrapped[0].starts_with('•'));
    assert!(wrapped[1].starts_with("  "));
}

#[test]
fn rejects_names_that_can_escape_their_destination() {
    for unsafe_name in [
        "",
        ".",
        "..",
        "../resume",
        "folder/resume",
        r"folder\resume",
    ] {
        assert!(!is_safe_file_stem(unsafe_name), "{unsafe_name:?} is unsafe");
    }
    for safe_name in ["resume", "resume-2026", "resume_en"] {
        assert!(is_safe_file_stem(safe_name), "{safe_name:?} should be safe");
    }
}

#[test]
fn reports_semantically_invalid_themes_as_theme_errors() {
    let invalid = include_str!("../assets/themes/classic/theme.yml")
        .replace("name: classic", "name: ../outside");
    let error = parse_theme_content("unsafe", &invalid).expect_err("theme should be rejected");
    assert_eq!(error.exit_code(), 5);
    assert!(error.to_string().contains("theme name"));
}
