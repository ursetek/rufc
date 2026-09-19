# DR immutability policy

Status: **PROPOSED**

## Context

URSE DR deliberately does not standardize whether accepted DRs may be
edited. Without project-level rules, accepted decisions can be silently
modified, breaking the audit trail and making it impossible to know
what was actually decided at a given point in time.

## Decision

1.  **Only PROPOSED DRs are mutable.** A DR with status `PROPOSED` may
    be edited, renamed, renumbered, or deleted freely during its
    development.

2.  **Non-PROPOSED DRs are immutable.** A DR with status `ACCEPTED`,
    `DEPRECATED`, `SUPERSEDED`, or `REJECTED` must not be edited except
    for:

    *   Fixing typos and grammatical errors that do not change meaning.
    *   Updating broken links (URLs, file paths, cross-references to
        other DRs).
    *   Correcting formatting issues that do not affect content.

3.  **Changes require a new DR.** To change, extend, or reverse a
    non-PROPOSED decision, create a new DR with a new number. The new
    DR's `Context` section must explicitly reference the old DR and
    explain why the change is needed. If the new DR supersedes the old
    one, update the old DR's status to `SUPERSEDED` and add a reference
    to the new DR.

4.  **Status transitions.** Status changes follow this lifecycle:

    *   `PROPOSED` -> `ACCEPTED` (decision approved)
    *   `PROPOSED` -> `REJECTED` (decision not adopted)
    *   `ACCEPTED` -> `DEPRECATED` (no longer recommended but still
        valid)
    *   `ACCEPTED` -> `SUPERSEDED` (replaced by new DR)
    *   `DEPRECATED` -> `SUPERSEDED` (explicitly replaced)
    *   `REJECTED` -> `PROPOSED` (reconsideration with new number)

    Reverse transitions (e.g., `ACCEPTED` -> `PROPOSED`) are not
    permitted. A decision that needs revision requires a new DR.

5.  **Deletion is prohibited.** Non-PROPOSED DRs must not be deleted.
    If a decision is no longer relevant, mark it `DEPRECATED` or
    `SUPERSEDED` instead.

## Consequences

*   Accepted decisions form a stable, auditable record of project
    evolution.
*   The rationale behind past decisions remains accessible even when
    the decision is later changed.
*   Breaking changes to the project's design are explicit and
    traceable through the DR chain.
*   `git log` and `git blame` remain meaningful for understanding the
    history of decisions.
*   Automated tools can rely on the stability of non-PROPOSED DRs for
    cross-references and compliance checking.

## References

*   `docs/dr/000-dr.md` (URSE DR adoption)
*   [URSE DR standard](https://ursetek.github.io/standards/standards/dr/index.html)
