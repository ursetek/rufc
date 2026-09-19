# Diagnostics system

Status: **PROPOSED**

## Context

DR 004 divided diagnostics into three classes (content errors,
configuration errors, warnings) with different handling policies. This
approach proved too rigid: the classification mixed concerns (what went
wrong vs. how to handle it), and the fixed policy per class did not
allow users to tune verbosity or severity for specific conditions.

A more flexible system is needed where each diagnostic condition can be
independently configured, while still preserving the hard requirement
that certain failures must prevent output generation.

## Considered options

*   **Keep DR 004.** Three fixed classes with fixed policies. Simple but
    inflexible.
*   **Severity levels only.** Every diagnostic has a severity (info,
    warning, error, fatal). Too many levels, unclear mapping to user
    intent.
*   **Lints-style configuration.** Each diagnostic has a slug, a default
    level, and can be overridden per-project or per-invocation. Proven
    in Rust (`clippy`), flexible, user-friendly.

## Decision

Option 3. Lints-style diagnostics with three levels and a special
category for fatal failures.

### 1. Diagnostic categories

Every diagnostic condition belongs to a **category** that describes
what part of the system reported it:

*   `config` &mdash; `Rufc.toml` parsing and validation.
*   `meta` &mdash; `toml,meta` block parsing and validation.
*   `links` &mdash; wikilink resolution and cross-reference validation.
*   `i18n` &mdash; translation processing and locale handling.
*   `structure` &mdash; source directory structure and resource files.
*   `io` &mdash; file system operations and I/O errors.

Categories are informational. They do not affect handling policy.

### 2. Diagnostic levels

Every diagnostic condition has one of three **levels**:

*   `ignore` &mdash; the condition is not reported. Build continues as
    if the condition did not occur.
*   `warn` &mdash; the condition is reported but does not affect the
    build outcome. The site is written normally, and the process exits
    with code zero.
*   `deny` &mdash; the condition is reported as an error. If the build
    can continue, diagnostics are accumulated and the process exits with
    a non-zero code at the end without writing output. If the build
    cannot continue (see fatal diagnostics below), the process exits
    immediately.

### 3. Default levels

Each diagnostic condition has a **default level** defined in the
documentation. Common defaults:

*   `ignore`: cosmetic issues, style violations, optional features not
    used.
*   `warn`: missing optional files, orphan resources, deprecated
    features used.
*   `deny`: broken links, malformed configuration, missing required
    fields, translation structure mismatch.

The complete list of diagnostic slugs and their default levels is
maintained in the user documentation.

### 4. Configuration

Diagnostic levels are determined in the following order of precedence
(highest to lowest):

1.  **Explicit CLI flags** (`-I`, `-W`, `-D` for specific slugs).
2.  **Mode flags** (`--pedantic`, `--relaxed`).
3.  **`Rufc.toml` settings**.
4.  **Default levels**.

**Explicit CLI flags.** Each level has a corresponding flag that applies
to a specific diagnostic slug:

*   `-I <slug>` / `--ignore <slug>` &mdash; set slug to `ignore`.
*   `-W <slug>` / `--warn <slug>` &mdash; set slug to `warn`.
*   `-D <slug>` / `--deny <slug>` &mdash; set slug to `deny`.

Multiple flags can be specified. They are processed in order, so later
flags override earlier ones for the same slug.

**Mode flags.** Two mode flags provide bulk transformations:

*   `--pedantic` &mdash; promotes all `warn` diagnostics to `deny`,
    except those explicitly overridden by `-I`, `-W`, or `-D` flags or
    configured in `Rufc.toml`. This mode is recommended for CI and
    strict validation.
*   `--relaxed` &mdash; demotes all `warn` diagnostics to `ignore`,
    except those explicitly overridden by `-I`, `-W`, or `-D` flags or
    configured in `Rufc.toml`. This mode is not recommended for
    production use, as it may hide issues that should be addressed.

Mode flags cannot be used together. If both are specified, the command
fails with a configuration error.

**`Rufc.toml`.** The `diagnostics` table maps diagnostic slugs to levels:

```toml
[diagnostics]
orphan-resource-folder = "warn"
missing-translation = "ignore"
broken-wikilink = "deny"
```

### 5. Fatal diagnostics

Some diagnostic conditions are **fatal**: they prevent the build from
producing correct output. Fatal diagnostics are always at level `deny`
and cannot be overridden to `warn` or `ignore` by any mechanism
(`Rufc.toml`, CLI flags, mode flags).

Fatal diagnostics are divided into two subcategories:

**Immediate termination.** These conditions make it impossible to
continue processing. When encountered, the process prints only this
diagnostic (discarding any previously accumulated diagnostics) and
exits immediately with a non-zero code. Examples:

*   `Rufc.toml` syntax error.
*   Missing required configuration field.
*   Source and output paths overlap.
*   I/O error preventing source tree reading.

**Accumulated termination.** These conditions allow processing to
continue for error collection, but the build cannot produce output.
Diagnostics are accumulated, and at the end of the build, all
accumulated diagnostics are printed and the process exits with a
non-zero code without writing output. Examples:

*   Broken wikilink.
*   Translation heading structure mismatch.
*   Missing required `general.created-at` field.

### 6. Diagnostic rendering

All diagnostics are rendered using `annotate-snippets`, even when the
diagnostic does not have a specific code position. This ensures
consistent formatting across all diagnostic types.

Rendering options (Unicode box-drawing, ANSI colors) follow the same
rules as DR 004: enabled by default, disabled when `-A` / `--ascii` is
passed or when the output descriptor is not a TTY.

Every diagnostic includes:

*   Diagnostic slug (e.g., `broken-wikilink`).
*   Level (`warn` or `deny`).
*   Category.
*   Short description.
*   Source location when applicable (file path, line, column, offset).
*   Hint when a fix is obvious.

### 7. Build outcome

The build outcome depends on accumulated diagnostics:

*   **No `deny` diagnostics.** All `warn` diagnostics are printed. The
    site is written. Exit code is zero.
*   **Only accumulated `deny` diagnostics.** All diagnostics (both
    `warn` and `deny`) are printed. No site is written. Exit code is
    non-zero.
*   **Immediate termination `deny` diagnostic.** Only this diagnostic is
    printed. No site is written. Exit code is non-zero.

## Consequences

*   Users can tune diagnostic verbosity and severity per-condition.
*   The distinction between "what went wrong" (category) and "how to
    handle it" (level) is explicit.
*   Fatal diagnostics guarantee that broken builds never produce output.
*   The diagnostic system is extensible: new conditions can be added
    without changing the core policy.
*   Configuration in `Rufc.toml` allows project-wide defaults.
*   CLI flags allow per-invocation overrides for CI or debugging.
*   `--pedantic` mode enables strict validation suitable for CI.
*   `--relaxed` mode reduces noise but may hide issues.
*   `annotate-snippets` provides consistent formatting across all
    diagnostic types.
*   Immediate termination diagnostics discard accumulated diagnostics to
    avoid noise when a fundamental error makes them irrelevant.
*   This DR supersedes DR 004.

## References

*   `docs/dr/004-error-reporting.md` (superseded by this DR)
*   `docs/dr/008-configuration-schema.md` (adds `diagnostics` table)
