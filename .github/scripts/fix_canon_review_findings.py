from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    if old not in text:
        raise SystemExit(f"Expected block not found in {path}: {old[:120]!r}")
    file.write_text(text.replace(old, new, 1))


ts = "apps/desktop/src/CanonReview.tsx"
replace_once(
    ts,
    '''  const [entryKind, setEntryKind] = useState<CanonEntryKind>("RULE");
  const [canonicalStatus, setCanonicalStatus] =
    useState<CanonicalStatus>("ALPHA_DA_TESTARE");''',
    '''  const [entryKind, setEntryKind] = useState<CanonEntryKind | "">("");
  const [canonicalStatus, setCanonicalStatus] = useState<
    CanonicalStatus | ""
  >("");''',
)
replace_once(
    ts,
    '''    setEntryKind("RULE");
    setCanonicalStatus("ALPHA_DA_TESTARE");''',
    '''    setEntryKind("");
    setCanonicalStatus("");''',
)
replace_once(
    ts,
    '''  const approve = async () => {
    const request: ApproveCanonDraftsRequest = {''',
    '''  const approve = async () => {
    if (!entryKind || !canonicalStatus) {
      setError("Seleziona esplicitamente categoria e stato canonico.");
      return;
    }
    const request: ApproveCanonDraftsRequest = {''',
)
replace_once(
    ts,
    '''                    >
                      {canonEntryKinds.map((kind) => (''',
    '''                    >
                      <option value="" disabled>
                        Seleziona categoria
                      </option>
                      {canonEntryKinds.map((kind) => (''',
)
replace_once(
    ts,
    '''                    >
                      {canonicalStatuses.map((status) => (''',
    '''                    >
                      <option value="" disabled>
                        Seleziona stato
                      </option>
                      {canonicalStatuses.map((status) => (''',
)
replace_once(
    ts,
    '''                      !title.trim() ||
                      !normalizedText.trim() ||''',
    '''                      !title.trim() ||
                      !entryKind ||
                      !canonicalStatus ||
                      !normalizedText.trim() ||''',
)

rust = "apps/desktop/src-tauri/src/canon_review.rs"
replace_once(
    rust,
    '''    let import_runs: HashSet<&str> = drafts
        .iter()
        .map(|draft| draft.import_run_id.as_str())
        .collect();
    if import_runs.len() != 1 {
        return Err(review_error(
            "drafts from different import runs cannot be merged into one canon entry",
        ));
    }
    Ok(())
}
''',
    '''    Ok(())
}

fn validate_same_import_run(drafts: &[CanonReviewDraftItem]) -> Result<(), AppError> {
    let import_runs: HashSet<&str> = drafts
        .iter()
        .map(|draft| draft.import_run_id.as_str())
        .collect();
    if import_runs.len() != 1 {
        return Err(review_error(
            "drafts from different import runs cannot be merged into one canon entry",
        ));
    }
    Ok(())
}
''',
)
replace_once(
    rust,
    '''    let drafts = load_drafts_by_ids(pool, &draft_ids).await?;
    validate_reviewable_drafts(&draft_ids, &drafts)?;
    let now = Utc::now().to_rfc3339();
    let joined_ids = drafts''',
    '''    let drafts = load_drafts_by_ids(pool, &draft_ids).await?;
    validate_reviewable_drafts(&draft_ids, &drafts)?;
    validate_same_import_run(&drafts)?;
    let now = Utc::now().to_rfc3339();
    let joined_ids = drafts''',
)
replace_once(
    rust,
    '''    #[tokio::test]
    async fn reviewed_drafts_cannot_be_reviewed_twice() {''',
    '''    #[test]
    fn cross_run_drafts_are_reviewable_but_not_mergeable() {
        let draft = |id: &str, import_run_id: &str| CanonReviewDraftItem {
            id: id.into(),
            import_run_id: import_run_id.into(),
            raw_block_id: format!("block-{id}"),
            source_anchor: format!("sc://test/{id}"),
            source_part: "word/document.xml".into(),
            block_index: 0,
            block_kind: "PARAGRAPH".into(),
            style_name: None,
            original_text: id.into(),
            text_sha256: "a".repeat(64),
            review_status: "PENDING_HUMAN_REVIEW".into(),
            canonical_status: None,
        };
        let drafts = vec![draft("a", "run-a"), draft("b", "run-b")];
        let ids = drafts
            .iter()
            .map(|draft| draft.id.clone())
            .collect::<Vec<_>>();
        assert!(validate_reviewable_drafts(&ids, &drafts).is_ok());
        assert!(validate_same_import_run(&drafts).is_err());
    }

    #[tokio::test]
    async fn reviewed_drafts_cannot_be_reviewed_twice() {''',
)

test = "apps/desktop/src/CanonReview.test.tsx"
replace_once(
    test,
    '''  expect(screen.getByText("1 elementi visibili")).toBeInTheDocument();
});''',
    '''  expect(screen.getByText("1 elementi visibili")).toBeInTheDocument();
  expect(screen.getByLabelText("Categoria")).toHaveValue("");
  expect(screen.getByLabelText("Stato canonico")).toHaveValue("");
  expect(
    screen.getByRole("button", { name: "Approva nel canon" }),
  ).toBeDisabled();
});''',
)
