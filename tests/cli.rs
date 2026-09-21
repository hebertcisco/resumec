use std::fs;
use std::path::PathBuf;
use std::process::Command;

use tempfile::tempdir;
use zip::ZipArchive;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn parses_json_yaml_and_toml_via_validate() {
    for fixture in [
        "examples/resume.yaml",
        "tests/fixtures/resume.json",
        "tests/fixtures/resume.toml",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_resumec"))
            .args(["validate", fixture, "--json-output"])
            .current_dir(repo_root())
            .output()
            .expect("validate command should run");
        assert!(
            output.status.success(),
            "{} failed: {}",
            fixture,
            String::from_utf8_lossy(&output.stderr)
        );
        let payload: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
        assert_eq!(payload["status"], "success");
    }
}

#[test]
fn returns_structured_errors_for_invalid_resume() {
    let output = Command::new(env!("CARGO_BIN_EXE_resumec"))
        .args([
            "validate",
            "tests/fixtures/invalid_resume.yaml",
            "--json-output",
        ])
        .current_dir(repo_root())
        .output()
        .expect("validate command should run");
    assert_eq!(output.status.code(), Some(1));
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(payload["status"], "error");
    assert!(payload["errors"].as_array().expect("errors array").len() >= 2);
}

#[test]
fn build_json_output_creates_pdf_and_docx() {
    let temp = tempdir().expect("temp dir");
    let output = Command::new(env!("CARGO_BIN_EXE_resumec"))
        .args([
            "build",
            "examples/resume.yaml",
            "--format",
            "both",
            "--theme",
            "modern",
            "--output-dir",
            temp.path().to_str().expect("utf-8 output dir"),
            "--output-name",
            "resume-test",
            "--json-output",
            "--overwrite",
        ])
        .current_dir(repo_root())
        .output()
        .expect("build command should run");
    assert!(
        output.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(payload["status"], "success");
    let outputs = payload["data"]["outputs"]
        .as_array()
        .expect("outputs array");
    assert_eq!(outputs.len(), 2);
    let docx_path = temp.path().join("resume-test.docx");
    let pdf_path = temp.path().join("resume-test.pdf");
    assert!(pdf_path.exists());
    assert!(docx_path.exists());
    let file = fs::File::open(docx_path).expect("docx should exist");
    let mut archive = ZipArchive::new(file).expect("docx should be a zip archive");
    let mut document_xml = archive
        .by_name("word/document.xml")
        .expect("document.xml should exist");
    let mut content = String::new();
    use std::io::Read;
    document_xml
        .read_to_string(&mut content)
        .expect("document.xml should be readable");
    assert!(content.contains("Ana Souza"));
}

#[test]
fn build_json_output_requires_overwrite_for_existing_files() {
    let temp = tempdir().expect("temp dir");
    let existing = temp.path().join("resume-test.pdf");
    fs::write(&existing, b"existing").expect("should create existing file");

    let output = Command::new(env!("CARGO_BIN_EXE_resumec"))
        .args([
            "build",
            "examples/resume.yaml",
            "--format",
            "pdf",
            "--output-dir",
            temp.path().to_str().expect("utf-8 output dir"),
            "--output-name",
            "resume-test",
            "--json-output",
        ])
        .current_dir(repo_root())
        .output()
        .expect("build command should run");

    assert_eq!(output.status.code(), Some(3));
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(payload["status"], "error");
    assert!(
        payload["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .any(|error| error["field"]
                .as_str()
                .unwrap_or_default()
                .ends_with("resume-test.pdf"))
    );
}

#[test]
fn build_rejects_output_names_with_path_components() {
    let temp = tempdir().expect("temp dir");
    let output = Command::new(env!("CARGO_BIN_EXE_resumec"))
        .args([
            "build",
            "examples/resume.yaml",
            "--output-dir",
            temp.path().to_str().expect("utf-8 output dir"),
            "--output-name",
            "../outside",
            "--json-output",
        ])
        .current_dir(repo_root())
        .output()
        .expect("build command should run");

    assert_eq!(output.status.code(), Some(7));
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(payload["status"], "error");
    assert!(!temp.path().join("../outside.pdf").exists());
}
