<!--
Title & commit message format (Conventional Commits) — the auto-label
workflow labels this PR from the title prefix:

  feat:     new tool/feature        -> enhancement
  fix:      bug fix                 -> bugfix
  docs:     documentation only      -> documentation
  chore:    maintenance             -> maintenance
  refactor: no behavior change      -> refactor
  test:     tests only              -> testing
  ci:       CI/DevOps               -> devops

A scope is welcome: `feat(tools): implement validate_task_completion`.
Use the same format for commit messages: `<type>(<scope>): <imperative summary>`.
-->

## Summary

<!-- What does this PR do and why? 1-3 sentences. -->

## Related issues

<!-- e.g. Closes #11 — required for feature work so issues close on merge -->

## Changes

<!-- Bullet list of what changed. -->

## Verification

<!-- Check everything you ran locally (same gate as CI): -->

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings`
- [ ] `cargo test --workspace --locked`
- [ ] New tool inputs/outputs derive `JsonSchema` (if applicable)
- [ ] No arbitrary filesystem access — `SafeRepositoryFs` only (if applicable)
