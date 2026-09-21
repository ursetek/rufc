//! Document module tests.

use rufc::document::{Document, DocumentError};

fn valid_document() -> &'static str {
    r#"```toml,meta
[general]
created-at = "2026-09-18"
```

# Test Document

This is the body content.
"#
}

#[test]
fn parse_valid_document() {
    let doc = Document::parse_str(valid_document(), "0001").unwrap();
    assert_eq!(doc.number, "0001");
    assert_eq!(doc.title, Some("Test Document".to_string()));
    assert_eq!(
        doc.metadata.general.created_at.as_deref(),
        Some("2026-09-18")
    );
    assert!(!doc.metadata.general.deprecated);
    assert!(doc.metadata.general.keywords.is_empty());
}

#[test]
fn parse_document_with_bom() {
    let content = format!("\u{feff}{}", valid_document());
    let doc = Document::parse_str(&content, "0001").unwrap();
    assert_eq!(doc.title, Some("Test Document".to_string()));
}

#[test]
fn parse_document_with_html_comments_before_meta() {
    let content = r#"<!-- This is a comment -->

```toml,meta
[general]
created-at = "2026-09-18"
```

# Test Document
"#;
    let doc = Document::parse_str(content, "0001").unwrap();
    assert_eq!(doc.title, Some("Test Document".to_string()));
}

#[test]
fn missing_meta_block_is_error() {
    let content = r"# Test Document

No metadata block here.
";
    let result = Document::parse_str(content, "0001");
    assert!(matches!(result, Err(DocumentError::MissingMetaBlock)));
}

#[test]
fn yaml_frontmatter_is_error() {
    let content = r"---
title: Test
---

# Test Document
";
    let result = Document::parse_str(content, "0001");
    assert!(matches!(result, Err(DocumentError::YamlFrontmatter)));
}

#[test]
fn content_before_meta_block_is_error() {
    let content = r#"Some text before the block.

```toml,meta
[general]
created-at = "2026-09-18"
```

# Test Document
"#;
    let result = Document::parse_str(content, "0001");
    assert!(matches!(result, Err(DocumentError::MissingMetaBlock)));
}

#[test]
fn multiple_meta_blocks_is_error() {
    let content = r#"```toml,meta
[general]
created-at = "2026-09-18"
```

# Title

```toml,meta
[general]
created-at = "2026-09-19"
```
"#;
    let result = Document::parse_str(content, "0001");
    assert!(matches!(result, Err(DocumentError::MultipleMetaBlocks)));
}

#[test]
fn missing_created_at_is_error() {
    let content = r"```toml,meta
[general]
deprecated = false
```

# Test Document
";
    let result = Document::parse_str(content, "0001");
    assert!(matches!(result, Err(DocumentError::MissingCreatedAt)));
}

#[test]
fn invalid_toml_syntax_is_error() {
    let content = r#"```toml,meta
[general
created-at = "2026-09-18"
```

# Test Document
"#;
    let result = Document::parse_str(content, "0001");
    assert!(matches!(result, Err(DocumentError::TomlParse(_))));
}

#[test]
fn extract_headings_multiple_levels() {
    let content = r#"```toml,meta
[general]
created-at = "2026-09-18"
```

# Main Title

Some paragraph.

## Section One

More content.

### Subsection

Even more.

## Section Two
"#;
    let doc = Document::parse_str(content, "0001").unwrap();
    assert_eq!(doc.headings.len(), 4);
    assert_eq!(doc.headings[0].level, 1);
    assert_eq!(doc.headings[0].text, "Main Title");
    assert_eq!(doc.headings[0].position, 1);
    assert_eq!(doc.headings[1].level, 2);
    assert_eq!(doc.headings[1].text, "Section One");
    assert_eq!(doc.headings[1].position, 2);
    assert_eq!(doc.headings[2].level, 3);
    assert_eq!(doc.headings[2].text, "Subsection");
    assert_eq!(doc.headings[2].position, 3);
    assert_eq!(doc.headings[3].level, 2);
    assert_eq!(doc.headings[3].text, "Section Two");
    assert_eq!(doc.headings[3].position, 4);
}

#[test]
fn document_without_h1_has_no_title() {
    let content = r#"```toml,meta
[general]
created-at = "2026-09-18"
```

## Only Section Heading

No main title here.
"#;
    let doc = Document::parse_str(content, "0001").unwrap();
    assert_eq!(doc.title, None);
    assert_eq!(doc.headings.len(), 1);
    assert_eq!(doc.headings[0].level, 2);
}

#[test]
fn metadata_with_optional_fields() {
    let content = r#"```toml,meta
[general]
created-at = "2026-09-18"
deprecated = true
keywords = ["standards", "process"]
see-also = ["0010", "0011"]
related = ["002-SUPERSEDED", "003"]
authors = ["Ivan Chetchasov <vi.is.chapmann@gmail.com>"]
```

# Test Document
"#;
    let doc = Document::parse_str(content, "0001").unwrap();
    assert!(doc.metadata.general.deprecated);
    assert_eq!(doc.metadata.general.keywords.len(), 2);
    assert_eq!(doc.metadata.general.keywords[0], "standards");
    assert_eq!(doc.metadata.general.see_also.len(), 2);
    assert_eq!(doc.metadata.general.see_also[0], "0010");
    assert_eq!(doc.metadata.general.related.len(), 2);
    assert_eq!(doc.metadata.general.related[0], "002-SUPERSEDED");
    assert_eq!(doc.metadata.general.authors.len(), 1);
}

#[test]
fn metadata_with_custom_fields() {
    let content = r#"```toml,meta
[general]
created-at = "2026-09-18"

[team]
owner = "someone"
reviewers = ["alice", "bob"]
```

# Test Document
"#;
    let doc = Document::parse_str(content, "0001").unwrap();
    assert!(doc.metadata.custom.contains_key("team"));
}

#[test]
fn body_preserves_content_after_meta() {
    let content = r#"```toml,meta
[general]
created-at = "2026-09-18"
```

# Test Document

First paragraph.

## Section

Second paragraph with [[0002]] wikilink.
"#;
    let doc = Document::parse_str(content, "0001").unwrap();
    assert!(doc.body.contains("First paragraph."));
    assert!(doc.body.contains("[[0002]]"));
    assert!(!doc.body.contains("toml,meta"));
}

#[test]
fn heading_with_inline_formatting() {
    let content = r#"```toml,meta
[general]
created-at = "2026-09-18"
```

# Title with *emphasis* and `code`
"#;
    let doc = Document::parse_str(content, "0001").unwrap();
    assert_eq!(doc.headings.len(), 1);
    // pulldown-cmark extracts text content, not formatting
    assert!(doc.headings[0].text.contains("Title with"));
    assert!(doc.headings[0].text.contains("emphasis"));
    assert!(doc.headings[0].text.contains("code"));
}
