from pathlib import Path
import json


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    if old not in text:
        raise SystemExit(f"Expected block not found in {path}: {old[:120]!r}")
    file.write_text(text.replace(old, new, 1))


def replace_all(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    if old not in text:
        raise SystemExit(f"Expected text not found in {path}: {old!r}")
    file.write_text(text.replace(old, new))


lib = "apps/desktop/src-tauri/src/lib.rs"
replace_once(
    lib,
    "mod canon_import;\nmod canon_review;\nmod database_studio;",
    "mod canon_import;\nmod canon_review;\nmod cloud;\nmod database_studio;",
)
replace_once(
    lib,
    "use database_studio::{",
    "use cloud::{\n    CloudSettingsUpdate, CloudStatus, get_cloud_status as load_cloud_status,\n    update_cloud_settings as save_cloud_settings,\n};\nuse database_studio::{",
)
replace_once(
    lib,
    "    #[error(\"database studio error: {0}\")]\n    DatabaseStudio(String),",
    "    #[error(\"database studio error: {0}\")]\n    DatabaseStudio(String),\n    #[error(\"cloud configuration error: {0}\")]\n    Cloud(String),",
)
replace_all(lib, '"Phase 1.5"', '"Phase 1.6"')
replace_once(
    lib,
    '        "Database Studio".into(),\n    ]',
    '        "Database Studio".into(),\n        "Supabase cloud configuration".into(),\n        "Local sync outbox foundation".into(),\n    ]',
)
replace_once(
    lib,
    "#[tauri::command]\nasync fn upsert_database_review_note(\n    app: tauri::AppHandle,\n    update: CanonReviewNoteUpdate,\n) -> Result<DatabaseAuditEntry, AppError> {\n    let pool = open_app_pool(&app).await?;\n    save_review_note(&pool, update).await\n}\n\npub fn run()",
    "#[tauri::command]\nasync fn upsert_database_review_note(\n    app: tauri::AppHandle,\n    update: CanonReviewNoteUpdate,\n) -> Result<DatabaseAuditEntry, AppError> {\n    let pool = open_app_pool(&app).await?;\n    save_review_note(&pool, update).await\n}\n\n#[tauri::command]\nasync fn get_cloud_status(app: tauri::AppHandle) -> Result<CloudStatus, AppError> {\n    let pool = open_app_pool(&app).await?;\n    load_cloud_status(&pool).await\n}\n\n#[tauri::command]\nasync fn update_cloud_settings(\n    app: tauri::AppHandle,\n    update: CloudSettingsUpdate,\n) -> Result<CloudStatus, AppError> {\n    let pool = open_app_pool(&app).await?;\n    save_cloud_settings(&pool, update).await\n}\n\npub fn run()",
)
replace_once(
    lib,
    "            update_database_project_metadata,\n            upsert_database_review_note\n",
    "            update_database_project_metadata,\n            upsert_database_review_note,\n            get_cloud_status,\n            update_cloud_settings\n",
)
replace_once(
    lib,
    '            "Review imported drafts and build the approved canon registry before publishing the Living Codex."\n                .into(),',
    '            "Configure the optional Supabase workspace, then continue approving canon before publishing the Living Codex."\n                .into(),',
)

app = "apps/desktop/src/App.tsx"
replace_once(
    app,
    'import { CanonReview } from "./CanonReview";\nimport { DatabaseStudio } from "./DatabaseStudio";',
    'import { CanonReview } from "./CanonReview";\nimport { CloudSync } from "./CloudSync";\nimport { DatabaseStudio } from "./DatabaseStudio";',
)
replace_all(app, 'developmentStage: "Phase 1.5"', 'developmentStage: "Phase 1.6"')
replace_once(
    app,
    '    "Database Studio",\n  ],',
    '    "Database Studio",\n    "Cloud & Sync",\n  ],',
)
replace_once(
    app,
    '  nextRecommendedPhase:\n    "Esegui l\'app desktop Tauri per accedere al database SQLite locale.",',
    '  nextRecommendedPhase:\n    "Esegui l\'app desktop Tauri per accedere a SQLite e alla configurazione Supabase opzionale.",',
)
replace_once(
    app,
    '<p className="phase-badge">Phase 1.5 · Canon Review</p>',
    '<p className="phase-badge">Phase 1.6 · Cloud Foundation</p>',
)
replace_once(
    app,
    '        {health && active === "Database Studio" && <DatabaseStudio />}\n        {health &&\n',
    '        {health && active === "Database Studio" && <DatabaseStudio />}\n        {health && active === "Cloud & Sync" && <CloudSync />}\n        {health &&\n',
)
replace_once(
    app,
    '          active !== "Canon Review" &&\n          active !== "Database Studio" && <NotImplemented title={active} />}',
    '          active !== "Canon Review" &&\n          active !== "Database Studio" &&\n          active !== "Cloud & Sync" && <NotImplemented title={active} />}',
)

replace_once(
    "packages/ui/src/index.ts",
    '  "Database Studio",\n  "Codex",',
    '  "Database Studio",\n  "Cloud & Sync",\n  "Codex",',
)
replace_once(
    "apps/desktop/src/main.tsx",
    'import "./density.css";',
    'import "./density.css";\nimport "./cloud.css";',
)

package_path = Path("apps/desktop/package.json")
package = json.loads(package_path.read_text())
package["version"] = "0.4.0"
package["dependencies"]["@supabase/supabase-js"] = "2.110.7"
package_path.write_text(json.dumps(package, indent=2) + "\n")

root_package_path = Path("package.json")
root_package = json.loads(root_package_path.read_text())
root_package["scripts"]["supabase:schema:check"] = "node scripts/validate-supabase-schema.mjs"
root_package["scripts"]["check"] = root_package["scripts"]["check"] + " && pnpm supabase:schema:check"
root_package_path.write_text(json.dumps(root_package, indent=2) + "\n")

replace_all("apps/desktop/src-tauri/Cargo.toml", 'version = "0.3.2"', 'version = "0.4.0"')
replace_all("apps/desktop/src-tauri/tauri.conf.json", '"version": "0.3.2"', '"version": "0.4.0"')
