# Samples PRD

Overview: Samples demonstrate integration patterns for supported languages and serve as smoke tests for CI.

Goals:
- Ship at least one working sample per language (C, Kotlin, TypeScript, Rust variants).
- Provide authoritative Quickstart documented in repo README.
- Automate smoke tests for each sample in CI.

Acceptance Criteria:
- Each sample has a README with run steps and automated smoke test that validates end-to-end behavior.
- Quickstart reproduces on a fresh environment within 10 minutes.

Maintenance:
- Keep samples minimal and CI-validated to prevent rot.

PRD version: 0.1
