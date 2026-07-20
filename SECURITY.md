# Security

Shadow Council Studio is local-first. Sprint 0 uses local SQLite and local files only.

Do not commit secrets, API keys, tokens, local databases or private environment files. Imported files must be treated as untrusted binary/source material and never executed.

Report dependency vulnerabilities in a private issue or direct owner channel while the project remains private.

No telemetry, analytics, external accounts, authentication or cloud dependency are permitted in Sprint 0.

Future work: if optional AI workflows are approved later, provider keys must be isolated in OS-protected secret storage and never synced or logged.
