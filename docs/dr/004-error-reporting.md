# Error reporting and build failure policy

Status: **SUPERSEDED** by `docs/dr/009-diagnostics-system.md`

## Context

This decision record has been superseded by DR 009, which introduces a
more flexible lints-style diagnostic system with configurable levels
and categories.

The previous design divided diagnostics into three classes (content
errors, configuration errors, warnings) with fixed handling policies.
This approach proved too rigid: the classification mixed concerns (what
went wrong vs. how to handle it), and the fixed policy per class did
not allow users to tune verbosity or severity for specific conditions.

See `docs/dr/009-diagnostics-system.md` for the current specification.

## References

*   `docs/dr/009-diagnostics-system.md` (supersedes this DR)
