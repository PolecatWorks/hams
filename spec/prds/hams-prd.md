# hams/ Product Requirements Document

Overview: hams/ is the core library implementing primary functionality and public API surface. This PRD focuses on API stability, testing, and packaging.

Goals:
- Stable, documented public API with semver guarantees.
- ≥80% unit/integration coverage for core logic.
- Reproducible builds for releases (pinned toolchains, Docker builders).

Key User Stories:
- As an Integrator, I need a stable API and clear changelog to upgrade safely.
- As a Maintainer, I need tests and CI to validate changes across platforms.

Acceptance Criteria:
- API documented in README and generated reference.
- CI runs unit/integration tests and measures coverage.
- Release produces a versioned crate with deterministic build.

Risks/Mitigations:
- Algorithm regressions: add property-based tests.
- Performance regressions: add benchmarks and CI alerts.

PRD version: 0.1
