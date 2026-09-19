# Configuration schema

Status: **PROPOSED**

## Context

Rufc requires project-wide configuration in `Rufc.toml`. This DR defines
the schema for this configuration file, including fields that affect how
documents are processed and rendered.

## Decision

`Rufc.toml` contains the following top-level tables:

*   `general` &mdash; core configuration fields.
*   `diagnostics` &mdash; diagnostic level overrides (see DR 009).

### `general.source.path`

A string specifying the directory containing documents and resources.
This path is resolved relative to the directory containing `Rufc.toml`.
A trailing slash is accepted but optional.

Default: `"src/"`.

Defined in DR 002.

### `general.modifiers`

A table defining the vocabulary of relationship modifiers used in
`general.related` fields in document metadata. Keys are modifier
identifiers (ASCII alphanumeric, case-sensitive). Values are arrays of
exactly two strings: the forward form and the reverse form.

Example:

```toml
[general.modifiers]
SUPERSEDED = ["Supersedes", "Superseded by"]
BROKEN = ["Breaks", "Broken by"]
```

When a document declares `related = ["0002-SUPERSEDED"]`, the forward
form "Supersedes" is used on the source document's page, and the
reverse form "Superseded by" is used on document 0002's page to
indicate the reverse relationship.

Default: `{}`.

Defined in DR 002.

### `general.linker`

A non-negative integer index into the `general.linkers` array. This
linker is used to resolve number-only wikilink targets (e.g.,
`[[0001]]`).

Default: `0`.

Defined in DR 003.

### `general.linkers`

An array of tables, each defining a linker for wikilink resolution.
Each table has two required string fields:

| Key | Type | Meaning |
|---|---|---|
| `from` | string | Template for matching wikilink targets. Must contain exactly one `{}` placeholder. |
| `to` | string | Template for generating URLs. Must contain exactly one `{}` placeholder. |

The `{}` placeholder in both templates matches one or more ASCII digits
(`0-9`). Everything outside `{}` is matched literally and
case-sensitively.

Example:

```toml
[[general.linkers]]
from = "URSE RFC {}"
to = "https://ursetek.github.io/rfcs/{}"

[[general.linkers]]
from = "URSE DR {}"
to = "https://ursetek.github.io/standards/{}"
```

Duplicate `from` templates are a configuration error per DR 009.

Required field.

Defined in DR 003.

### `diagnostics`

A table mapping diagnostic slugs to levels. Keys are diagnostic slugs
(strings). Values are one of `"ignore"`, `"warn"`, or `"deny"`.

Example:

```toml
[diagnostics]
orphan-resource-folder = "warn"
missing-translation = "ignore"
broken-wikilink = "deny"
```

Settings in this table override default levels but can be overridden by
CLI flags (`-I`, `-W`, `-D`) and mode flags (`--pedantic`, `--relaxed`).

Default: `{}`.

Defined in DR 009.

## Consequences

*   `Rufc.toml` is the single source of project-wide configuration.
*   The `general.modifiers` table defines the relationship vocabulary
    for the entire project.
*   The `general.linkers` array defines how wikilinks are matched and
    resolved.
*   Unknown keys under `general` are configuration errors per DR 009.
*   Missing required fields (`general.linkers`) are configuration
    errors per DR 009.
*   All configuration errors are diagnostic conditions with default
    level `deny` per DR 009.

## References

*   `docs/dr/002-document-source-resources.md` (defines `source.path` and `modifiers`)
*   `docs/dr/003-wikilink-format.md` (defines `linker` and `linkers`)
*   `docs/dr/007-configuration-location.md` (defines where `Rufc.toml` is found)
*   `docs/dr/009-diagnostics-system.md` (defines diagnostic configuration)
