# Project Goals: Rufc

## Purpose

Rufc publishes a registry of formal documents as a static site. It
understands the document format &mdash; numbered files, status metadata,
cross-references &mdash; and produces stable URLs, an index, and resolved
links. It does not try to be a general-purpose site generator: every design
decision favors a document registry, and every capability that does not
serve that goal is a non-goal.

## Problem

Existing static site generators solve adjacent problems:

*   Hugo is general-purpose. Its content model is a page tree, not a
    numbered registry; forcing it into one requires inventing conventions
    outside the tool.
*   mdBook is built for linear books. Documents in a registry are
    independent; ordering is by number, not by chapter.
*   Jekyll brings a Ruby runtime and a plugin ecosystem. A registry
    generator should be a single binary with no runtime dependencies.
*   Astro is a framework. Rufc is not a framework; it is a tool.

None of them understand the shape of the content: a fixed set of numbered
documents, each with metadata, each referring to others by number.

## Scope

Rufc reads a directory of Markdown documents and produces a static site:

*   One directory per document, one page per document, one index page for
    the registry.
*   Metadata extracted from the document itself (title, status, authors,
    date) drives the index table and the document page.
*   Cross-references between documents are parsed and resolved into
    hyperlinks. A reference to a non-existent document is an error.
*   The output is a static site &mdash; HTML, CSS, JavaScript, static
    assets &mdash; with no server-side component.

## Non-goals

*   Server-side rendering. All rendering happens at build time. Nothing
    runs on the host after deployment.
*   Server-side search. If search is offered, the index is a static file
    consumed by client-side JavaScript.
*   Incremental builds. Every `build` renders the full site. There is no
    cache between runs.
*   Content versioning. Rufc does not read VCS metadata, does not render
    diffs, does not track document history.
*   Blog posts, changelogs, galleries. Rufc is for formal documents.

## Users

Rufc is a public tool. It is published on crates.io, follows Semantic
Versioning, and accepts issues and pull requests from anyone.

The first user is URSETek, whose RFC and DR repositories are the driving
use case. URSETek requirements shape the roadmap. External users are
welcome but do not drive design: when a feature request conflicts with
the URSETek use case, the URSETek use case wins.

Because the tool is public, changes to the configuration schema, the CLI,
and the output structure are breaking changes and follow SemVer. A
pre-1.0 release may break on any version bump; a post-1.0 release may
break only on a major.

## Success criteria

A user opens a terminal in a repository that holds numbered Markdown
documents, adds `text/0001-foo.md`, commits, and pushes. A GitHub Action
runs Rufc. Two minutes later the registry site shows the new document in
the index, the document page is reachable at a stable URL, and every
cross-reference from the new document to existing ones resolves into a
working link.

When this scenario works end-to-end, Rufc is functional in its first
approximation.

## Status

Pre-alpha. Version 0.1.0. Nothing is implemented yet. The document format
that Rufc expects, the directory layout of a registry, and the rules for
cross-references are decisions still to be recorded in `docs/dr/`.
