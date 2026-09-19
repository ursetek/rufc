# Module structure

Status: **PROPOSED**

## Context

The Rufc library must be organized so that:

*   each module has one responsibility, nameable in a single phrase;
*   dependencies between modules form a directed acyclic graph;
*   the library exposes enough surface for the CLI and integration tests,
    but no more;
*   the structure is navigable by a reader who knows the domain
    (documents, registries, links) but not the implementation.

The URSETek standard requires every application to be distributed as a
library crate plus a binary crate. The binary contains only the CLI; all
logic lives in the library.

## Decision

Feature-first. The library has the following top-level modules:

*   `config` &mdash; loads and validates `Rufc.toml`. Owns the
    configuration schema. No dependencies on other Rufc modules.
*   `document` &mdash; parses one Markdown file into a `Document` value:
    title, metadata, body. Knows the document format. No dependencies on
    other Rufc modules.
*   `registry` &mdash; discovers documents in a directory, holds them,
    assigns numbers. Depends on `config` and `document`.
*   `links` &mdash; parses cross-reference syntax and resolves it against
    the registry. Depends on `config` and `registry`.
*   `render` &mdash; turns a document and a resolved link set into HTML.
    Depends on `config`, `document`, `links`.
*   `site` &mdash; orchestrates the build and check: reads config, builds the
    registry, resolves links, renders pages, writes output (for build).
    Exposes `build` and `check` entry points. Depends on every other
    module.
*   `diagnostics` &mdash; diagnostic conditions, categories, levels, and
    accumulation logic. Depended on by every other module; depends on
    nothing. Defines the diagnostic system specified in DR 009.

Dependencies flow strictly downward. `site` depends on `render`, `links`,
`registry`, `config`. `render` depends on `document`, `links`, `config`.
`links` depends on `registry`, `config`. `registry` depends on `document`,
`config`. `config` and `document` are leaves. `diagnostics` is orthogonal
and depended on by all modules. No cycles.

The binary `src/main.rs` contains argument parsing and a single call into
`site`. It contains no logic that could live in the library.

## Consequences

*   Module names map to concepts, not to implementation steps. A reader
    who knows the domain can navigate the source.
*   Adding a module requires justifying that it does not fit an existing
    one. The set above is not a floor; a future `theme` module for user
    template loading is a plausible addition and would sit between
    `render` and `site`.
*   Cycles are prohibited. When a new dependency would create one, the
    design is wrong: either the modules are not what they claim to be, or
    the dependency belongs elsewhere.
*   The library is testable at each layer. `document::parse` is a pure
    function from bytes to a value. `registry::from_dir` is testable with
    a temporary directory. `links::resolve` is testable with an in-memory
    registry. `render` is testable with fixed inputs.
*   The public/private split is decided per module, not here. The default
    is `pub(crate)`; a module's `pub` items are those the CLI or the
    integration tests need.
*   The `diagnostics` module owns the diagnostic system from DR 009:
    condition definitions, categories, levels, accumulation, and
    rendering. All other modules emit diagnostics through this module's
    API.

## References

*   `docs/project-goals.md`
*   `docs/dr/009-diagnostics-system.md`
