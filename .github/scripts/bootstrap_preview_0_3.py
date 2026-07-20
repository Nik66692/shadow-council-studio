from pathlib import Path
import json


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    if old not in text:
        raise SystemExit(f"Expected block not found in {path}: {old!r}")
    file.write_text(text.replace(old, new, 1))


tauri_path = Path("apps/desktop/src-tauri/tauri.conf.json")
tauri = json.loads(tauri_path.read_text())
tauri["version"] = "0.3.0"
tauri_path.write_text(json.dumps(tauri, indent=2, ensure_ascii=False) + "\n")

package_path = Path("apps/desktop/package.json")
package = json.loads(package_path.read_text())
package["version"] = "0.3.0"
package_path.write_text(json.dumps(package, indent=2) + "\n")

replace_once(
    "apps/desktop/src-tauri/Cargo.toml",
    'version = "0.2.0"',
    'version = "0.3.0"',
)

readme_path = Path("README.md")
readme = readme_path.read_text()
old_preview = """## Windows Preview 0.2

Preview 0.2 packages the Phase 1 canonical importer and Database Studio as an unsigned local-first Windows NSIS installer. The installed application uses the immutable bundled canon manifest and Source of Truth v1.3; development builds may use repository resources. Windows may show SmartScreen because internal previews are not code-signed.
"""
new_preview = """## Windows Preview 0.3

Preview 0.3 packages the deterministic canonical importer, Database Studio and controlled Canon Review as an unsigned local-first Windows NSIS installer. It includes explicit approval and rejection, ordered source provenance, the approved canon registry and immutable decision history. The installed application uses the bundled manifest and Source of Truth v1.3; development builds may use repository resources. Windows may show SmartScreen because internal previews are not code-signed.
"""
if old_preview not in readme:
    raise SystemExit("Expected Preview 0.2 README section not found")
readme_path.write_text(readme.replace(old_preview, new_preview, 1))
