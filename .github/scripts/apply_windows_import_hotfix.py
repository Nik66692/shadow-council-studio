from pathlib import Path
import json


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    if old not in text:
        raise SystemExit(f"Expected block not found in {path}: {old[:160]!r}")
    file.write_text(text.replace(old, new, 1))


cargo = Path("apps/desktop/src-tauri/Cargo.toml")
cargo_text = cargo.read_text().replace('version = "0.3.0"', 'version = "0.3.1"', 1)
dependency_anchor = 'chrono = { version = "0.4", features = ["serde"] }\n'
if 'zip = ' not in cargo_text:
    cargo_text = cargo_text.replace(
        dependency_anchor,
        dependency_anchor
        + 'zip = { version = "8.6.0", default-features = false, features = ["deflate"] }\n',
        1,
    )
cargo.write_text(cargo_text)

package_path = Path("apps/desktop/package.json")
package = json.loads(package_path.read_text())
package["version"] = "0.3.1"
package_path.write_text(json.dumps(package, indent=2, ensure_ascii=False) + "\n")

tauri_path = Path("apps/desktop/src-tauri/tauri.conf.json")
tauri = json.loads(tauri_path.read_text())
tauri["version"] = "0.3.1"
tauri_path.write_text(json.dumps(tauri, indent=2, ensure_ascii=False) + "\n")

workflow_path = Path(".github/workflows/windows-preview.yml")
workflow_path.write_text(
    workflow_path.read_text().replace(
        "shadow-council-studio-windows-preview-0.3.0",
        "shadow-council-studio-windows-preview-0.3.1",
    )
)

rust = "apps/desktop/src-tauri/src/canon_import.rs"
replace_once(
    rust,
    'use std::{path::Path, process::Command};',
    'use std::{fs::File, io::Read, path::Path};',
)
replace_once(
    rust,
    '''#[cfg(target_os = "windows")]
fn extract_document_xml(source_path: &Path) -> Result<String, AppError> {
    let script = r#"
param([string]$docx)
Add-Type -AssemblyName System.IO.Compression.FileSystem
$archive = [System.IO.Compression.ZipFile]::OpenRead($docx)
try {
  $entry = $archive.GetEntry('word/document.xml')
  if ($null -eq $entry) { throw 'word/document.xml not found' }
  $reader = New-Object System.IO.StreamReader($entry.Open(), [System.Text.Encoding]::UTF8, $true)
  try { $reader.ReadToEnd() } finally { $reader.Dispose() }
} finally {
  $archive.Dispose()
}
"#;
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .arg(source_path)
        .output()?;
    if !output.status.success() {
        return Err(AppError::CanonManifest(format!(
            "PowerShell could not read {DOCUMENT_XML_PART}: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| AppError::CanonManifest(format!("invalid UTF-8 in document.xml: {error}")))
}

#[cfg(not(target_os = "windows"))]
fn extract_document_xml(source_path: &Path) -> Result<String, AppError> {
    let output = Command::new("unzip")
        .arg("-p")
        .arg(source_path)
        .arg(DOCUMENT_XML_PART)
        .output()?;
    if !output.status.success() {
        return Err(AppError::CanonManifest(format!(
            "unzip could not read {DOCUMENT_XML_PART}: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| AppError::CanonManifest(format!("invalid UTF-8 in document.xml: {error}")))
}
''',
    '''fn extract_document_xml(source_path: &Path) -> Result<String, AppError> {
    let file = File::open(source_path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|error| {
        AppError::CanonManifest(format!("invalid DOCX ZIP archive: {error}"))
    })?;
    let mut entry = archive.by_name(DOCUMENT_XML_PART).map_err(|error| {
        AppError::CanonManifest(format!(
            "DOCX entry {DOCUMENT_XML_PART} is unavailable: {error}"
        ))
    })?;
    let mut xml = String::new();
    entry.read_to_string(&mut xml).map_err(|error| {
        AppError::CanonManifest(format!(
            "could not read {DOCUMENT_XML_PART} as UTF-8: {error}"
        ))
    })?;
    Ok(xml)
}
''',
)
replace_once(
    rust,
    '''    #[test]
    fn parser_preserves_text_and_warns_when_tables_are_flattened() {''',
    '''    #[test]
    fn native_docx_reader_extracts_document_xml_without_external_tools() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../../docs/canon/source/v1.3/Shadow_Council_Source_of_Truth_v1.3.docx",
        );
        let xml = extract_document_xml(&source).unwrap();
        assert!(xml.contains("<w:document"));
        assert!(xml.contains("<w:body"));
    }

    #[test]
    fn parser_preserves_text_and_warns_when_tables_are_flattened() {''',
)

app = "apps/desktop/src/App.tsx"
replace_once(
    app,
    '''const emptyReview: CanonImportReviewSnapshot = {
  run: null,
  drafts: [],
  warnings: [],
  importedNow: false,
};
''',
    '''const emptyReview: CanonImportReviewSnapshot = {
  run: null,
  drafts: [],
  warnings: [],
  importedNow: false,
};

export function messageFromError(cause: unknown, fallback: string): string {
  if (cause instanceof Error && cause.message.trim()) return cause.message;
  if (typeof cause === "string" && cause.trim()) return cause;
  if (cause && typeof cause === "object") {
    const message = (cause as { message?: unknown }).message;
    if (typeof message === "string" && message.trim()) return message;
    try {
      const serialized = JSON.stringify(cause);
      if (serialized && serialized !== "{}") return serialized;
    } catch {
      // Fall through to the caller-provided message.
    }
  }
  return fallback;
}
''',
)
replace_once(
    app,
    '''      setError(
        cause instanceof Error
          ? cause.message
          : "Importazione canonica non riuscita.",
      );''',
    '''      setError(messageFromError(cause, "Importazione canonica non riuscita."));''',
)

test = "apps/desktop/src/App.test.tsx"
replace_once(
    test,
    'import { App } from "./App";',
    'import { App, messageFromError } from "./App";',
)
test_path = Path(test)
test_path.write_text(
    test_path.read_text()
    + '''\n\nit("surfaces string-shaped Tauri errors", () => {
  expect(messageFromError("canonical source failed", "fallback")).toBe(
    "canonical source failed",
  );
  expect(messageFromError({ message: "zip failed" }, "fallback")).toBe(
    "zip failed",
  );
});\n'''
)

readme = Path("README.md")
readme.write_text(
    readme.read_text()
    + "\n\n## Windows Preview 0.3.1\n\nHotfix 0.3.1 replaces external PowerShell/unzip DOCX extraction with an in-process Rust ZIP reader and surfaces detailed Tauri import errors. Canonical content remains unchanged.\n"
)
