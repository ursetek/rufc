# Rufc

A static site generator for numbered document registries.

Rufc understands the shape of formal document collections: numbered files,
status metadata, cross-references. It produces stable URLs, an index page, and
resolved links. It is not a general-purpose site generator; every design
decision favors a document registry.

> ***Status***
>
> This project is in pre-alpha now.

## Why Rufc?

Existing static site generators solve adjacent problems:

*   **Hugo** is general-purpose. Its content model is a page tree, not a
    numbered registry.
*   **mdBook** is built for linear books. Documents in a registry are
    independent; ordering is by number, not by chapter.
*   **Jekyll** brings a Ruby runtime and a plugin ecosystem. A registry
    generator should be a single binary with no runtime dependencies.
*   **Astro** is a framework. Rufc is not a framework; it is a tool.

None of them understand the shape of the content: a fixed set of numbered
documents, each with metadata, each referring to others by number.

## Installation

There is possibility to install Rufc using Cargo:

```bash
cargo install rufc
```

## Quick Start

Create a new registry:

```bash
rufc init
```

This creates:

```text
./
|-- Rufc.toml
|-- src/
|   |-- common/
|   `-- 0000.md
`-- .gitignore
```

Add a document, then build:

```bash
rufc build
```

The output is written to `public/` by default.

## Document Format

Each document is a Markdown file with a TOML metadata block as the first content
element:

````markdown
```toml,meta
[general]
created-at = "2026-09-18"
keywords = ["standards", "process"]
see-also = ["0010", "0011"]
```

# Document Title

Content goes here. Cross-reference other documents with wikilinks:
[[0002]], [[URSE DOC 0003#2|section 2 of DOC 0003]].
````

Required metadata fields:

| Key | Type | Meaning |
|---|---|---|
| `general.created-at` | string | ISO 8601 date (`YYYY-MM-DD`) |

Optional fields include `general.deprecated`, `general.keywords`,
`general.see-also`, `general.related`, `general.authors`, and
`general.i18n` for localization.

## Cross-References

Wikilinks use the syntax `[[target#anchor|alias]]`:

*   `[[0001]]` — link to document 0001
*   `[[0001#2]]` — link to the second heading of document 0001
*   `[[0001|custom text]]` — link with custom display text
*   `[[URSE DOC 0001]]` — link through a configured linker template

Linkers are defined in `Rufc.toml` and map target patterns to URLs.

## Configuration

Configuration lives in `Rufc.toml`:

````toml
[general]
source.path = "src/"
linker = 0

[[general.linkers]]
from = "URSE DOC {}"
to = "https://example.com/docs/{}"

[general.modifiers]
SUPERSEDED = ["Supersedes", "Superseded by"]

[diagnostics]
orphan-resource-folder = "warn"
````

See `docs/dr/008-cfg-schema.md` for the complete configuration schema.

## CLI

```text
rufc [-r <root>] build [-o <output>] [-A] [-I <slug>]... [-W <slug>]... [-D <slug>]... [--pedantic|--relaxed]
rufc [-r <root>] check [-A] [-I <slug>]... [-W <slug>]... [-D <slug>]... [--pedantic|--relaxed]
rufc [-r <root>] init [-A]
rufc new <name> [-A]
```

### Commands

*   `build` — Build the static site from source documents
*   `check` — Validate the source without producing output
*   `init` — Create the registry structure in the current directory
*   `new <name>` — Create a directory and initialize a registry inside it

### Diagnostic Flags

*   `-I <slug>` — Set diagnostic slug to `ignore` level
*   `-W <slug>` — Set diagnostic slug to `warn` level
*   `-D <slug>` — Set diagnostic slug to `deny` level
*   `--pedantic` — Promote all `warn` diagnostics to `deny`
*   `--relaxed` — Demote all `warn` diagnostics to `ignore`
*   `-A` / `--ascii` — Disable Unicode box-drawing and ANSI colors

## Localization

Translations live in `src/i18n/NNNN/<locale>.md`:

```text
src/
|-- 0001.md           # Base document (English)
`-- i18n/
    `-- 0001/
        |-- ru_RU.md  # Russian translation
        `-- de_DE.md  # German translation
```

Per-locale metadata additions are specified in the base document's
`general.i18n` table.

## Development

```bash
# Format code
just fmt

# Run linter
just lint

# Run tests
just test

# Run full CI checks
just check

# Build release binary
just build
```

Requires Rust 1.85+ (edition 2024).

## License

MIT OR Apache-2.0
