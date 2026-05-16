# CI/CD & Releases PRD

Overview: CI/CD provides PR validation, cross-language matrices, and reproducible releases.

Goals:
- GitHub Actions matrix builds across OS/toolchain versions.
- Lint, unit, integration, and smoke tests for PRs; failures block merges.
- Release automation: create changelog, build artifacts, sign/tag, and publish to targets.

Acceptance Criteria:
- PRs run full checks; maintainers can run release job that produces deterministic artifacts.
- Secrets stored in GitHub Actions; Dependabot and SCA enabled.

Targets (TBD): crates.io, npm, Maven Central, Docker Hub.

PRD version: 0.1
