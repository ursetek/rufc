# CLI interface

Status: **PROPOSED**

## Context

Rufc is a command-line tool. This DR defines the complete CLI
interface, including all commands, flags, and their interactions.

## Decision

### Project location flags

These flags specify the project root and apply to `build`, `check`, and
`init` commands. They must appear before the command name:

*   `-r <path>` / `--root <path>` &mdash; specifies the directory
    containing `Rufc.toml`. If not provided, the current working
    directory is used. Defined in DR 007. For `init`, specifies the
    directory where the registry structure will be created.

### Diagnostic flags

These flags control diagnostic levels per DR 009 and apply to all
commands that produce diagnostics:

*   `-I <slug>` / `--ignore <slug>` &mdash; set diagnostic slug to
    `ignore` level.
*   `-W <slug>` / `--warn <slug>` &mdash; set diagnostic slug to
    `warn` level.
*   `-D <slug>` / `--deny <slug>` &mdash; set diagnostic slug to
    `deny` level.
*   `--pedantic` &mdash; promote all `warn` diagnostics to `deny`,
    except those explicitly overridden.
*   `--relaxed` &mdash; demote all `warn` diagnostics to `ignore`,
    except those explicitly overridden.
*   `-A` / `--ascii` &mdash; disable Unicode box-drawing and ANSI
    colors in diagnostic output.

Multiple diagnostic flags can be specified and are processed in order.
`--pedantic` and `--relaxed` are mutually exclusive.

Diagnostic levels are determined in this order of precedence (highest
to lowest):

1.  Explicit CLI flags (`-I`, `-W`, `-D`).
2.  Mode flags (`--pedantic`, `--relaxed`).
3.  `Rufc.toml` settings.
4.  Default levels.

### Commands

**`rufc build [-o <output>]`**

Builds the static site from the source directory specified in
`Rufc.toml` and writes it to `<output>`.

*   `-o <output>` / `--output <output>` is optional. If not provided,
    the default value `public` is used.
*   `<output>` is resolved relative to the current working directory
    unless absolute.
*   The output path and source path must be mutually exclusive: neither
    may be inside the other, and they may not be the same path. This
    check uses canonical paths after resolving symlinks.
*   The binary contains argument parsing only and delegates building to
    the library `site::build` entry point per DR 001.

**`rufc check`**

Validates the source directory without producing output. Performs all
the same checks as `build` (configuration parsing, document parsing,
link resolution, translation validation) but skips the output writing
phase.

*   Useful for CI pipelines that only need to validate correctness.
*   Faster than `build` since no files are written.
*   The `-o` flag is not accepted since no output is produced.
*   The binary delegates to the library `site::check` entry point.

**`rufc init`**

Creates the registry structure in the current working directory or the
directory specified by `--root`. Defined in DR 002.

**`rufc new <name>`**

Creates a directory `<name>` and performs the same scaffolding as
`rufc init` inside it. Defined in DR 002.

### Complete CLI form

```
rufc [-r <root>] build [-o <output>] [-A|--ascii] [-I <slug>]... [-W <slug>]... [-D <slug>]... [--pedantic|--relaxed]
rufc [-r <root>] check [-A|--ascii] [-I <slug>]... [-W <slug>]... [-D <slug>]... [--pedantic|--relaxed]
rufc [-r <root>] init [-A|--ascii] [-I <slug>]... [-W <slug>]... [-D <slug>]... [--pedantic|--relaxed]
rufc new <name> [-A|--ascii] [-I <slug>]... [-W <slug>]... [-D <slug>]... [--pedantic|--relaxed]
```

## Consequences

*   All commands support diagnostic configuration via CLI flags.
*   Project location flags (`-r`) must appear before the command name
    for `build`, `check`, and `init`.
*   Diagnostic flags can appear in any order after the command name.
*   `--pedantic` and `--relaxed` cannot be used together.
*   The `build` command uses `public` as the default output directory.
*   The `check` command performs validation without producing output.
*   Attempting to build into the source directory (or vice versa) is a
    diagnostic condition with default level `deny` and halts
    immediately.
*   The output path check uses canonical paths, so symlinks and
    relative path tricks cannot bypass the mutual exclusion.

## References

*   `docs/dr/001-module-structure.md`
*   `docs/dr/002-document-source-resources.md`
*   `docs/dr/007-configuration-location.md`
*   `docs/dr/009-diagnostics-system.md`
