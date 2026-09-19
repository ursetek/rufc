# Wikilink format

Status: **PROPOSED**

## Context

DR 002 fixed that cross-references between documents in body text use
wikilink syntax `[[target]]`, and that `general.see-also` and
`general.related` carry document numbers rather than wikilinks. This DR
fixes the grammar of `target`: what characters it may contain, how it is
parsed, how a target is matched against configuration, and how it is
resolved into a URL.

The pre-Rufc design used a fixed set of prefixes (`RFC`, `DR`, `ADR`)
matched against link targets. This DR replaces prefixes with templates:
a linker is a pair of templates, one for matching a target and one for
producing a URL. A linker whose matching template is `URSE RFC {}` and
whose URL template is `https://ursetek.github.io/rfcs/{}` behaves like
the old `RFC` prefix. A linker whose matching template is
`URSE RFC {} EXTENDED` supports targets that the old prefix model could
not express.

## Decision

### 1. Overall form

A wikilink is a sequence of characters beginning with `[[` and ending
with the next `]]`. No nesting: a `[[` inside a wikilink is not
interpreted. The content between the delimiters is the **link body**.

The link body has the form:

```
<target> [ "#" <anchor> ] [ "|" <alias> ]
```

A target, optionally followed by `#` and an anchor, optionally followed
by `|` and an alias. The three parts appear in this order. Any other
arrangement is not a wikilink; the text is left unchanged.

### 2. Target

The target identifies a document. A target is never empty. A wikilink
whose body begins with `#` or `|` is not recognized as a wikilink and
is left as literal text.

**Number-only form.** The target is a sequence of digits, at least one
digit. Leading zeros are permitted and are ignored: `[[1]]`,
`[[0001]]`, and `[[00000001]]` all resolve to document `0001`.

**Template form.** The target is matched against every entry in
`general.linkers`, in configuration order. A linker whose `from`
template contains `{}` matches a target when the template, with `{}`
replaced by one or more digits, is byte-identical to the target. The
first matching linker wins. If no linker matches, the target is a
diagnostic condition with default level `deny`.

Templates may place `{}` anywhere: `URSE RFC {}`, `URSE {} RFC`,
`URSE RFC {} EXTENDED`, `{}` alone. The `{}` placeholder matches ASCII
digits only. It does not match letters, whitespace, punctuation, or any
character outside `0-9`. Everything outside `{}` is matched literally
and is case-sensitively.

**Duplicate `from` templates are a configuration error.** If two
linkers have byte-identical `from` templates, the configuration is
rejected per DR 009.

**Default linker.** Configuration names one linker as the default:

```toml
[general]
linker = 0

[[general.linkers]]
from = "URSE RFC {}"
to = "https://ursetek.github.io/rfcs/{}"
```

`general.linker` is an integer index into `general.linkers`, default
`0`. A named key is not accepted: the value must be a non-negative
integer that is a valid index into the `general.linkers` array. A
number-only target is resolved through the default linker: the number
is substituted into the default linker's `from` template to produce a
canonical form, and the default linker's `to` template is used for the
URL.

### 3. Anchor

If present, the anchor is everything between `#` and the next `|` or
the end of the body.

The anchor is a **heading position**, expressed as a positive integer.
`#1` refers to the first `#`-level heading (the document title). `#2`
refers to the first `##`-level heading. `#3` refers to the second
`##`-level heading, and so on. Headings are numbered in document order,
across all levels, starting at 1.

An anchor that does not correspond to a heading is a diagnostic
condition with default level `deny`: either the position is greater
than the number of headings, or it is zero, or it is not an integer.

Heading text is never an anchor. Translating a document does not change
anchors, and does not change the URL fragment for any section.

### 4. Alias

If present, the alias is everything after the first `|` in the link
body. The alias is plain text. It may contain any character except `]]`;
`[`, `]`, `#`, and `|` are allowed and are not interpreted.

If the alias is absent, the rendered text is the canonical form
produced from the matched linker's `from` template. For `[[0001]]`
with the default linker above, the rendered text is `URSE RFC 0001`.
If the alias is present, the rendered text is the alias verbatim.

### 5. Resolution and rendering

A wikilink resolves to a link whose `href` is the target's URL, or,
when an anchor is present, the target's URL followed by `#section-<n>`,
where `<n>` is the anchor position.

The URL comes from the matched linker's `to` template, with `{}`
replaced by the number extracted from the target. A `to` template MUST
contain exactly one `{}`.

**URL structure.** The URL for a base document is the target number
alone. The URL for a translation is `<number>/<locale>`. This rule is
defined here and governs all wikilink resolution.

A broken wikilink is a diagnostic condition with default level `deny`.
Broken means any of:

*   The target's number matches no document in the registry.
*   The target matches no linker's `from` template.
*   The anchor is not a positive integer.
*   The anchor position exceeds the number of headings in the target.

### 6. Same-document links are forbidden

A wikilink without a target &mdash; `[[#2]]`, `[[|alias]]` &mdash; is
not recognized as a wikilink and is left as literal text.

Two reasons:

*   Plain Markdown already handles this: `[the section](#section-2)`
    links to a section of the current document. Reusing the same
    capability through wikilink syntax duplicates a solved problem.
*   A wikilink without a target reads as if it points somewhere else.
    A reader who is scanning a document sees `[[#2]]` and cannot tell
    that the link stays on the same page.

### 7. Localization

A wikilink in a translated document resolves the same way as a wikilink
in the base document. The rendered link points to the localized version
of the target when one exists for the current locale, and to the base
version otherwise.

An alias in a translated document is translated by the translator; Rufc
does not translate it. The target and the anchor are not translated.

### 8. Not applicable to `see-also` and `related`

Wikilinks are recognized in body text only. The `general.see-also` and
`general.related` fields use a document-number syntax defined in DR 002;
wikilink syntax has no meaning in those fields.

A `[[...]]` appearing as an entry of `see-also` or `related` is a
diagnostic condition with default level `deny`.

## Consequences

*   Wikilinks are recognized in body text of both base documents and
    translations.
*   A `[[` that does not have a matching `]]` before the end of the
    file is a diagnostic condition with default level `deny`.
*   A `[[` that contains a second `[[` before the closing `]]` is not a
    wikilink. The text is left as-is.
*   An empty target &mdash; `[[]]`, `[[#2]]`, `[[|alias]]` &mdash; is
    not a wikilink. The text is left as-is.
*   A target that matches no linker's `from` template is a diagnostic
    condition with default level `deny`.
*   A target whose number matches no document is a diagnostic condition
    with default level `deny`.
*   An anchor that does not correspond to a heading position is a
    diagnostic condition with default level `deny`.
*   A `to` template without exactly one `{}` is a configuration error
    per DR 009.
*   Two linkers with byte-identical `from` templates are a configuration
    error per DR 009.
*   `general.linker` must be a non-negative integer that is a valid
    index into `general.linkers`. Any other value is a configuration
    error per DR 009.
*   The rendered alias is HTML-escaped. `<`, `>`, `&`, `"`, and `'` in
    an alias become entities in the output.
*   A new configuration field is introduced: `general.linkers`, an
    array of tables, each with `from` and `to` fields (both strings).
    Required.
*   A new configuration field is introduced: `general.linker`, an
    integer index into `general.linkers`. Default `0`.
*   Two documents in one registry cannot share a number even if they
    would be reachable through different linkers. Numbers are unique
    across the registry.
*   The rendered URL for a base document is the number alone.
*   The rendered URL for a translation is `<number>/<locale>`.

## References

*   `docs/project-goals.md`
*   `docs/dr/001-module-structure.md`
*   `docs/dr/002-document-source-resources.md`
*   `docs/dr/009-diagnostics-system.md`
