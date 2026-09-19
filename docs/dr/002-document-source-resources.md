# Document format, source structure, arbitrary resources and localization

Status: **PROPOSED**

## Context

Four concerns are fixed together because each shapes the others: what a
single document file looks like, where documents live in the source
tree, how resources attached to documents are handled, and how
translations are stored alongside the base document.

The pre-Rufc draft split document metadata across an `## @rufc::meta`
section and a per-document `meta.toml` file, put relations in an
`## @rufc::rels` section, and reserved the `@` prefix for section names.
This DR consolidates all document and registry state into a single TOML
block, drops the `@` reservation, defines the localization layout, fixes
which metadata fields are required and which are optional, and fixes the
syntax of the fields that carry cross-references.

## Decision

### 1. Document format

A base document is a Markdown file with two structural elements.

**Metadata block.** The first content element of the file is a fenced
code block whose info string is exactly `toml,meta`. Its content is
TOML. Empty lines, HTML comments (`<!-- -->`), and a byte order mark
(BOM) before the block are ignored and are not considered content. YAML
frontmatter (`---` delimiters) is not recognized and is a diagnostic
condition with default level `deny` (see DR 009). No other content may
precede the block.

**Required fields.** Exactly one field is required, under the `general`
namespace:

| Key | Type | Meaning |
|---|---|---|
| `general.created-at` | string | ISO 8601 date (`YYYY-MM-DD`). |

A base document missing `general.created-at` is a diagnostic condition
with default level `deny` (see DR 009), naming the missing key and its
path.

**Optional fields.** All other Rufc-recognized fields are optional.
When absent, the default applies.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `general.deprecated` | boolean | `false` | Whether the document is deprecated. |
| `general.keywords` | array of strings | `[]` | Free-form tags. |
| `general.see-also` | array of strings | `[]` | Document numbers. Each entry is a string of ASCII digits. |
| `general.related` | array of strings | `[]` | Document numbers with optional modifiers: `NNNN` or `NNNN-MODIFIER`. |
| `general.authors` | array of strings | `[]` | Author identifiers. String format is provisional; see below. |
| `general.i18n` | table of tables | `{}` | Per-locale additions to the metadata. See below. |

**Provisional string format for `authors`.** Each entry is a free-form
string. Rufc does not parse, split, or validate it. The format may be
constrained, changed, or made mandatory in any release before 1.0.0,
and in any major release after 1.0.0.

**Cross-reference syntax in the body.** A cross-reference between
documents in body text is written as a wikilink: `[[target]]`. The
grammar of `target` is fixed in DR 003. Rufc recognizes wikilinks in
body text and resolves them at build time.

**`general.see-also`.** An array of strings. Each entry is a document
number: one or more ASCII digits. Leading zeros are permitted and
ignored on resolution. An entry that is not a string of digits is a
diagnostic condition with default level `deny`.

Rendered as a "See Also" block on the document page, listing the target
documents in the order they appear in the array. No reverse entries are
generated on target documents.

**`general.related`.** An array of strings. Each entry has the form
`NNNN` or `NNNN-MODIFIER`:

*   `NNNN` &mdash; a document number, same rules as `see-also` entries.
*   `NNNN-MODIFIER` &mdash; a document number, a single hyphen, and a
    modifier identifier registered in `Rufc.toml`'s `general.modifiers`
    table (see DR 008).

The modifier identifier consists of ASCII alphanumeric characters
(`A-Z`, `a-z`, `0-9`) and is matched case-sensitively against the keys
of `general.modifiers` in `Rufc.toml`. An unregistered modifier is a
diagnostic condition with default level `deny`. An entry with more than
one hyphen, or with a modifier containing non-alphanumeric characters,
is a diagnostic condition with default level `deny`.

Rendered as follows:

*   Entries without a modifier form a "Related" block on the document
    page, listing the targets.
*   Entries with modifier `X` form a block labelled with
    `general.modifiers.X[0]` (the forward form), listing the targets.
*   Each target document receives a reverse entry. For modifier `X`,
    the reverse form `general.modifiers.X[1]` labels the block on the
    target page; the source document is listed.

A target document may appear at most once in `general.related`, with or
without a modifier. Two entries that resolve to the same target are a
diagnostic condition with default level `deny`.

The distinction between `see-also` and `related` is deliberate:
`see-also` is a one-way recommendation, `related` is a declared
relationship with an optional type. Rufc does not infer one from the
other.

**`general.i18n`.** A table whose keys are locale tags and whose values
are tables mirroring the structure of `toml,meta`. For each locale, the
value is merged into the base `toml,meta` to produce the metadata as
seen in that locale.

Merge rules. For each key present in both the base and the locale
value:

*   **Both arrays.** The locale array is appended to the base array. The
    result is base entries followed by locale entries, in that order.
*   **Both tables.** The locale table is merged into the base table
    recursively. The same three rules apply at each level.
*   **Both scalars.** This is an unresolvable conflict. Rufc reports a
    diagnostic condition with default level `deny`, naming the key and
    both values.
*   **Type mismatch** (one is an array, the other a table or a scalar).
    This is a diagnostic condition with default level `deny`.

For each key present only in the locale value: the key is added to the
merged result as-is.

A `general.i18n` entry for a locale with no translation file at
`src/i18n/NNNN/<locale>.md` produces a diagnostic condition with
default level `warn`. The entry is otherwise processed as if the
translation file existed: the metadata is merged and is available to
templates. The translation is simply not rendered, because there is no
body to render.

A `general.i18n` entry whose structure does not mirror `toml,meta` at
every level is a diagnostic condition with default level `deny`.

`general.deprecated` and `general.created-at` describe the document
itself, not its presentation. A `general.i18n` path that reaches any of
them is a diagnostic condition with default level `deny`. A document's
deprecation state and creation date do not change with the reader's
language.

Example:

````markdown
```toml,meta
[general]
created-at = "2026-09-18"
deprecated = false
keywords = ["standards", "process"]
see-also = ["0010", "0011"]
related = ["002-SUPERSEDED", "003"]
authors = ["Ivan Chetchasov <vi.is.chapmann@gmail.com>"]

[general.i18n.ru_RU.general]
keywords = ["стандарты", "процесс"]
authors = ["McLover <lovermc@example.com> (переводчик)"]

[team]
owner = "someone"
```
````

**Arbitrary fields.** Keys outside the `general` namespace are user
fields. Rufc carries them through without interpretation; they are
available to themes and templates. There is no restriction on their
names or values, as long as they are valid TOML.

**Unknown keys under `general`.** A key under `general` that is not one
of the fields listed above is a diagnostic condition with default level
`deny`. The `general` namespace is reserved for Rufc; users who need
extra metadata use a namespace of their own.

**Body.** Everything after the metadata block. The first `#` heading in
the body is the document title. Any later `##` heading is a section.
Section levels are otherwise free: `###` and deeper may be used.

**Section names are not constrained.** Rufc does not reserve any prefix
or character in section names. A section named `@rufc::meta` in the
body has no special meaning.

The info string `toml,meta` is reserved. A base document with more than
one such block is a diagnostic condition with default level `deny`. A
`toml,meta` block anywhere other than the first content element of the
file is a diagnostic condition with default level `deny`.

### 2. Source structure

Documents live in a single directory, configurable as `source.path`,
default `src/`. Each document is a pair: a file `NNNN.md` and, when
resources exist, a folder `NNNN/`.

```text
src/
|-- common/
|   `-- ...
|-- 0000.md
|-- 0001.md
|-- 0001/
|   |-- diagram.png
|   `-- schema.svg
|-- 0002.md
`-- i18n/
    `-- ...
```

The document number `NNNN` is padded with leading zeros to four digits.
Four digits cover any realistic registry size and produce stable
lexical sorting. Numbers are unique across the registry. `0000` is
reserved for a template document and is ignored during builds. The file
`0000.md` is not required to exist. References to document `0000` in
`see-also`, `related`, or wikilinks are not permitted and are treated
as references to a non-existent document (diagnostic condition with
default level `deny`).

The folder `NNNN/` is optional. When present, it holds resources
attached to document `NNNN`. When absent, the document has no
resources. A folder `NNNN/` without a corresponding `NNNN.md` file
produces a diagnostic condition with default level `warn` and is
otherwise ignored.

### 3. Arbitrary resources

Per-document resources live in `src/NNNN/`. They are copied verbatim
to `<output>/NNNN/`. From Markdown, they are referenced by relative
path (`./diagram.png`). Rufc performs no rewriting and no validation on
these paths.

Site-wide resources live in `src/common/`. They are copied verbatim to
the output root.

`src/common/` is created by `rufc init` and `rufc new` as an empty
directory. The build command does not create it. Its contents are not
managed by Rufc.

Neither `src/NNNN/` nor the contents of `src/common/` are required.

**Locale-specific resources.** A translation has no resource folder of
its own. Its output `<output>/NNNN/la_CO.html` sits in the same
directory as the base document's resources, so any resource referenced
from a translation resolves through the base resource directory.

A resource that is specific to a locale lives in `src/NNNN/` alongside
the base resources, with the locale tag inserted before the extension.
Example: `preview.zh_CH.png` is the preview image used by the `zh_CH`
translation. Rufc does not interpret this naming; it is a convention
for authors, not a Rufc feature. Any file placed in `src/NNNN/` is
copied verbatim regardless of its name.

### 4. Localization

A translation of document `NNNN` into locale `xx_YY` is a file
`src/i18n/NNNN/xx_YY.md`. The locale tag uses an underscore between a
lowercase ISO 639-1 language code and an uppercase ISO 3166-1 alpha-2
region code. A tag without a region (`xx`) is also accepted.

The base language is English and is represented by `src/NNNN.md` with
no locale tag. A translation may not exist without its base document
(diagnostic condition with default level `deny`).

The output for a translation is `<output>/NNNN/xx_YY.html`. The base
document is rendered to `<output>/NNNN/index.html`.

**Translations carry no metadata.** A `toml,meta` block in a
translation is a diagnostic condition with default level `deny`.
Metadata lives in the base document only. Per-locale additions to
metadata are expressed through `general.i18n` in the base document, not
through a block in the translation.

**Heading structure must match the base document exactly.** A
translation must contain the same sequence of heading levels as the
base document, in the same order. `#`, `##`, `##` in the base requires
`#`, `##`, `##` in the translation. Heading text is free (it is
translated); heading level and position are not.

A translation with a different number of headings, different levels, or
a different order is a diagnostic condition with default level `deny`.
The diagnostic names the first heading that diverges. Adding or
omitting a section relative to the base is a diagnostic condition with
default level `deny`.

The body of a translation is otherwise free: paragraphs, lists, code
blocks, inline formatting, and cross-references may all differ from the
base.

### 5. Scaffolding commands

`rufc init` creates the registry structure in the current working
directory or the directory specified by `--root`. It creates `src/`,
`src/common/`, a `Rufc.toml` with minimal valid configuration (including
the required `general.linkers` field with at least one linker entry),
an empty `0000.md` file, and a `.gitignore` file containing the default
output directory path (`public/`). It does not overwrite existing files:
if a target file exists, the command reports the collision and exits
without modifying the tree.

`rufc new <name>` creates a directory `<name>` in the current working
directory and then performs the same scaffolding as `rufc init` inside
it. If `<name>` already exists and is not empty, the command reports
the collision and exits without modifying the tree.

Neither command creates any document other than the empty `0000.md`.
An empty registry has no documents; adding them is the author's task.

## Consequences

*   Discovery iterates `src/`, ignores `0000.md`, `common/`, `i18n/`,
    and any folder whose name is not a four-digit number, and treats
    each remaining `NNNN.md` as a base document. Each `NNNN/` folder is
    attached to the matching document.
*   A document without a `toml,meta` block is a diagnostic condition
    with default level `deny`.
*   An unknown key under `general` is a diagnostic condition with
    default level `deny`.
*   An entry in `general.see-also` that is not a string of ASCII digits
    is a diagnostic condition with default level `deny`.
*   An entry in `general.related` that contains more than one hyphen, or
    a modifier with non-alphanumeric characters, is a diagnostic
    condition with default level `deny`.
*   An entry in `general.related` whose modifier is not a key of
    `general.modifiers` in `Rufc.toml` is a diagnostic condition with
    default level `deny`.
*   Two entries in `general.related` that resolve to the same target are
    a diagnostic condition with default level `deny`.
*   A `toml,meta` block in a translation is a diagnostic condition with
    default level `deny`.
*   A translation without a base document is a diagnostic condition with
    default level `deny`.
*   A translation whose heading structure differs from the base is a
    diagnostic condition with default level `deny`.
*   A `general.i18n.<locale>` entry with no corresponding translation
    file produces a diagnostic condition with default level `warn`.
*   A `general.i18n.<locale>` entry that produces a scalar conflict or a
    type mismatch is a diagnostic condition with default level `deny`.
*   Anchor IDs in generated HTML are derived from heading position
    (`#section-1`, `#section-2`, ...), not from heading text, so a link
    to a section resolves to the corresponding section in every locale.
*   `rufc build` does not write to `src/`.
*   `rufc init` and `rufc new` refuse to overwrite existing files.
*   The config schema gains one field: `source.path`, string, default
    `"src/"`. A trailing slash is accepted but optional.
*   A document missing `general.created-at` is a diagnostic condition
    with default level `deny`.
*   A `general.i18n` path that reaches `general.deprecated` or
    `general.created-at` is a diagnostic condition with default level
    `deny`.
*   `general.deprecated` is optional and defaults to `false`. A document
    that does not mention deprecation is not deprecated.
*   The fields `general.linkers` and `general.linker` are introduced by
    DR 003. This DR does not define them.
*   The field `general.modifiers` is defined in DR 008 as part of
    `Rufc.toml`. This DR does not define it.
*   A folder `NNNN/` without a corresponding `NNNN.md` file produces a
    diagnostic condition with default level `warn` and is ignored.
*   References to document `0000` are diagnostic conditions with default
    level `deny`.
*   HTML comments and BOM before `toml,meta` are permitted; YAML
    frontmatter is a diagnostic condition with default level `deny`.
*   `rufc init` and `rufc new` create an empty `0000.md`, a minimal
    `Rufc.toml`, and a `.gitignore` containing the default output
    directory (`public/`).

## References

*   `docs/project-goals.md`
*   `docs/dr/001-module-structure.md`
*   `docs/dr/003-wikilink-format.md`
*   `docs/dr/008-configuration-schema.md`
*   `docs/dr/009-diagnostics-system.md`
