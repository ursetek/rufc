//! Document parsing (DR 002).
//!
//! Parses a single Markdown file into a `Document` value: title, metadata,
//! body. Knows the document format. No dependencies on other Rufc modules
//! (DR 001).

use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use serde::Deserialize;
use std::collections::HashMap;

/// Represents a heading in the document body.
#[derive(Debug, Clone)]
pub struct Heading {
    /// Heading level (1 = `#`, 2 = `##`, etc.).
    pub level: usize,
    /// Heading text content.
    pub text: String,
    /// Position in document order (1-based).
    pub position: usize,
}

/// Parsed document metadata from the `toml,meta` block.
#[derive(Debug, Clone, Deserialize)]
pub struct Metadata {
    /// The `[general]` namespace.
    pub general: GeneralMetadata,
    /// User-defined fields outside the `general` namespace.
    #[serde(flatten)]
    pub custom: HashMap<String, toml::Value>,
}

/// The `[general]` section of document metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct GeneralMetadata {
    /// ISO 8601 date (`YYYY-MM-DD`). Required.
    #[serde(default, rename = "created-at")]
    pub created_at: Option<String>,
    /// Whether the document is deprecated. Default: `false`.
    #[serde(default)]
    pub deprecated: bool,
    /// Free-form tags. Default: `[]`.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Document numbers for one-way recommendations. Default: `[]`.
    #[serde(default, rename = "see-also")]
    pub see_also: Vec<String>,
    /// Document numbers with optional modifiers. Default: `[]`.
    #[serde(default)]
    pub related: Vec<String>,
    /// Author identifiers. Default: `[]`.
    #[serde(default)]
    pub authors: Vec<String>,
    /// Per-locale metadata additions. Default: `{}`.
    #[serde(default)]
    pub i18n: HashMap<String, toml::Value>,
}

/// A parsed document.
#[derive(Debug, Clone)]
pub struct Document {
    /// Document number (e.g., `0001`).
    pub number: String,
    /// Parsed metadata from the `toml,meta` block.
    pub metadata: Metadata,
    /// Document title (first `#` heading in body).
    pub title: Option<String>,
    /// All headings in the body, in document order.
    pub headings: Vec<Heading>,
    /// Body content after the metadata block.
    pub body: String,
}

/// Errors that can occur during document parsing.
#[derive(Debug, thiserror::Error)]
pub enum DocumentError {
    /// I/O error when reading the file.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// YAML frontmatter detected (not supported).
    #[error("YAML frontmatter detected; use `toml,meta` code block instead")]
    YamlFrontmatter,
    /// Missing `toml,meta` block as the first content element.
    #[error("missing `toml,meta` block as the first content element")]
    MissingMetaBlock,
    /// Multiple `toml,meta` blocks found.
    #[error("multiple `toml,meta` blocks found; only one is allowed")]
    MultipleMetaBlocks,
    /// TOML syntax error in the metadata block.
    #[error("TOML parse error in `toml,meta` block: {0}")]
    TomlParse(#[from] toml::de::Error),
    /// Missing required `general.created-at` field.
    #[error("missing required field `general.created-at`")]
    MissingCreatedAt,
    /// Unknown key under `general` namespace.
    #[error("unknown key under `general`: {0}")]
    UnknownGeneralKey(String),
    /// Invalid `see-also` entry (not a string of digits).
    #[error("invalid `see-also` entry: {0}")]
    InvalidSeeAlsoEntry(String),
    /// Invalid `related` entry format.
    #[error("invalid `related` entry: {0}")]
    InvalidRelatedEntry(String),
}

impl Document {
    /// Parses a document from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, the metadata block is
    /// missing or malformed, or required fields are absent.
    pub fn parse(path: &std::path::Path, number: &str) -> Result<Self, DocumentError> {
        let content = std::fs::read_to_string(path)?;
        Self::parse_str(&content, number)
    }

    /// Parses a document from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the metadata block is missing or malformed,
    /// or required fields are absent.
    pub fn parse_str(content: &str, number: &str) -> Result<Self, DocumentError> {
        // Strip BOM if present
        let content = content.strip_prefix('\u{feff}').unwrap_or(content);

        // Find the `toml,meta` block
        let (meta_content, body) = Self::extract_meta_block(content)?;

        // Parse metadata
        let metadata: Metadata = toml::from_str(&meta_content)?;

        // Validate required fields
        if metadata.general.created_at.is_none() {
            return Err(DocumentError::MissingCreatedAt);
        }

        // Parse body for headings and title
        let headings = Self::extract_headings(&body);
        let title = headings
            .iter()
            .find(|h| h.level == 1)
            .map(|h| h.text.clone());

        Ok(Self {
            number: number.to_string(),
            metadata,
            title,
            headings,
            body,
        })
    }

    /// Extracts the `toml,meta` block from the document content.
    ///
    /// Returns the TOML content and the remaining body.
    fn extract_meta_block(content: &str) -> Result<(String, String), DocumentError> {
        let lines: Vec<&str> = content.lines().collect();
        let mut meta_start_line: Option<usize> = None;
        let mut meta_end_line: Option<usize> = None;
        let mut meta_count = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Allow empty lines and HTML comments before the block
            if meta_start_line.is_none() {
                if trimmed.is_empty() || trimmed.starts_with("<!--") || trimmed.ends_with("-->") {
                    continue;
                }

                // Check for YAML frontmatter
                if trimmed == "---" {
                    return Err(DocumentError::YamlFrontmatter);
                }

                // Check for `toml,meta` block start
                if trimmed.starts_with("```") && trimmed.contains("toml,meta") {
                    meta_count += 1;
                    if meta_count > 1 {
                        return Err(DocumentError::MultipleMetaBlocks);
                    }
                    meta_start_line = Some(i + 1);
                    continue;
                }

                // Any other content before the block is an error
                return Err(DocumentError::MissingMetaBlock);
            }

            // Inside the meta block: look for closing fence
            if meta_end_line.is_none() && trimmed == "```" {
                meta_end_line = Some(i);
            }

            // After the meta block: check for duplicate blocks
            if meta_end_line.is_some()
                && trimmed.starts_with("```")
                && trimmed.contains("toml,meta")
            {
                return Err(DocumentError::MultipleMetaBlocks);
            }
        }

        let start = meta_start_line.ok_or(DocumentError::MissingMetaBlock)?;
        let end = meta_end_line.ok_or(DocumentError::MissingMetaBlock)?;

        let meta_content = lines[start..end].join("\n");
        let body = if end + 1 < lines.len() {
            lines[end + 1..].join("\n")
        } else {
            String::new()
        };

        Ok((meta_content, body))
    }

    /// Extracts headings from the document body using `pulldown-cmark`.
    fn extract_headings(body: &str) -> Vec<Heading> {
        let parser = Parser::new(body);
        let mut headings = Vec::new();
        let mut current_heading: Option<(usize, String)> = None;
        let mut position = 0;

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    let level_num = level as usize;
                    position += 1;
                    current_heading = Some((level_num, String::new()));
                }
                Event::Text(text) | Event::Code(text) => {
                    if let Some((_, ref mut heading_text)) = current_heading {
                        heading_text.push_str(&text);
                    }
                }
                Event::End(TagEnd::Heading(_)) => {
                    if let Some((level, text)) = current_heading.take() {
                        headings.push(Heading {
                            level,
                            text,
                            position,
                        });
                    }
                }
                _ => {}
            }
        }

        headings
    }
}
