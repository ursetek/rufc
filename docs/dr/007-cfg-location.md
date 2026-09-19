# Configuration file location

Status: **PROPOSED**

## Context

Rufc needs to locate its configuration file (`Rufc.toml`) before it can
read any configuration, including the source path. This creates a
bootstrapping problem: the configuration file cannot live inside the
source directory it configures. The location must be determined by other
means.

## Considered options

*   **Fixed location.** Always in the current working directory.
    Simple but inflexible.
*   **Search upward.** Walk up the directory tree looking for
    `Rufc.toml`. Convenient but can accidentally pick up a parent
    project's configuration.
*   **CLI flag.** Explicit path via command line. Unambiguous but
    verbose.
*   **Environment variable.** `RUFC_ROOT` or similar. Adds another
    configuration surface.

## Decision

Configuration file location is resolved in this order:

1.  **CLI flag.** If `-r <path>` or `--root <path>` is provided, Rufc
    looks for `Rufc.toml` in `<path>`. If the file does not exist, the
    command fails with an error.

2.  **Current working directory.** If no CLI flag is provided, Rufc
    looks for `Rufc.toml` in the current working directory. If the file
    does not exist, the command fails with an error.

No upward search is performed. No environment variables are consulted.

The `source.path` configuration field specifies the directory containing
documents and resources, defaulting to `src/`. This path is resolved
relative to the directory containing `Rufc.toml`, not relative to the
current working directory.

## Consequences

*   `Rufc.toml` cannot live inside `source.path`. The configuration
    file is outside the source tree it configures.
*   Users working from a subdirectory of the project must use
    `--root` or `cd` to the project root.
*   The configuration location is deterministic and visible in the
    command line or the shell's current directory.
*   Accidental configuration pickup from parent directories is
    impossible.
*   `source.path` is always relative to the configuration file, not to
    the invocation point. This makes the configuration portable.

## References

*   `docs/dr/002-document-source-resources.md` (defines `source.path`)
*   `docs/dr/005-dir-cli.md` (defines CLI form)
