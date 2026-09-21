//! Registry: document discovery and storage (DR 002).
//!
//! Discovers documents in a directory, holds them, assigns numbers.
//! Depends on `config`, `document`, and `diagnostics` (DR 001).

use crate::config::RufcConfig;
use crate::diagnostics::{Category, Diagnostic, DiagnosticLevel, Diagnostics, SourceLocation};
use crate::document::Document;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A translation of a base document.
#[derive(Debug, Clone)]
pub struct Translation {
    /// Locale tag (e.g., `ru_RU`).
    pub locale: String,
    /// Document number this translation belongs to.
    pub number: String,
    /// Raw content of the translation file.
    pub content: String,
}

/// A document entry in the registry: base document plus translations and resources.
#[derive(Debug, Clone)]
pub struct RegistryEntry {
    /// The parsed base document.
    pub document: Document,
    /// Translations keyed by locale tag.
    pub translations: HashMap<String, Translation>,
    /// Whether the document has a resource folder (`src/NNNN/`).
    pub has_resources: bool,
}

/// The registry: a collection of all documents discovered in the source directory.
#[derive(Debug, Default)]
pub struct Registry {
    /// Documents keyed by their four-digit number (e.g., `0001`).
    pub entries: HashMap<String, RegistryEntry>,
}

/// Fatal errors that prevent the registry from being built at all.
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    /// I/O error when reading the source directory.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Source directory not found.
    #[error("source directory not found: {0}")]
    SourceNotFound(PathBuf),
}

/// Returns `true` if the path has a `.md` extension (case-insensitive).
#[must_use]
fn has_markdown_extension(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

impl Registry {
    /// Builds a registry by scanning the source directory.
    ///
    /// Non-fatal issues (parse errors, orphan folders, duplicate numbers)
    /// are emitted to `diagnostics` rather than returned as errors.
    ///
    /// # Errors
    ///
    /// Returns an error only for fatal conditions: I/O failures that
    /// prevent reading the source directory.
    pub fn build(
        config: &RufcConfig,
        config_dir: &Path,
        diagnostics: &mut Diagnostics,
    ) -> Result<Self, RegistryError> {
        let source_path = config.source_path(config_dir);

        if !source_path.exists() {
            return Err(RegistryError::SourceNotFound(source_path));
        }

        let mut registry = Self::default();

        // Discover base documents
        Self::discover_documents(&source_path, &mut registry, diagnostics)?;

        // Discover translations
        Self::discover_translations(&source_path, &mut registry, diagnostics)?;

        Ok(registry)
    }

    /// Discovers base documents (`NNNN.md`) in the source directory.
    fn discover_documents(
        source_path: &Path,
        registry: &mut Registry,
        diagnostics: &mut Diagnostics,
    ) -> Result<(), RegistryError> {
        let entries = std::fs::read_dir(source_path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Handle directories
            if path.is_dir() {
                // Check if it's a valid four-digit resource folder
                if Self::is_valid_document_number(&file_name) {
                    // Check if corresponding .md file exists
                    let md_path = source_path.join(format!("{file_name}.md"));
                    if !md_path.exists() {
                        diagnostics.emit(Diagnostic {
                            slug: "orphan-resource-folder".into(),
                            category: Category::Structure,
                            level: DiagnosticLevel::Warn,
                            message: format!(
                                "resource folder {file_name}/ has no corresponding {file_name}.md"
                            ),
                            location: Some(SourceLocation {
                                path: path.clone(),
                                line: 1,
                                column: 1,
                            }),
                            hint: Some(format!("create {file_name}.md or remove the folder")),
                            fatal: false,
                        });
                    }
                }
                // Skip `common/`, `i18n/`, and any non-four-digit folders
                continue;
            }

            // Skip `0000.md` (reserved template)
            if file_name == "0000.md" {
                continue;
            }

            // Only process `.md` files
            if !has_markdown_extension(&path) {
                continue;
            }

            // Extract the document number (file stem without extension)
            let number = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

            if !Self::is_valid_document_number(number) {
                continue;
            }

            // Check for duplicates before parsing
            if registry.entries.contains_key(number) {
                diagnostics.emit(Diagnostic {
                    slug: "duplicate-document-number".into(),
                    category: Category::Structure,
                    level: DiagnosticLevel::Deny,
                    message: format!("duplicate document number: {number}"),
                    location: Some(SourceLocation {
                        path: path.clone(),
                        line: 1,
                        column: 1,
                    }),
                    hint: None,
                    fatal: false,
                });
                continue;
            }

            // Parse the document
            let document = match Document::parse(&path, number) {
                Ok(doc) => doc,
                Err(e) => {
                    diagnostics.emit(Diagnostic {
                        slug: "document-parse-error".into(),
                        category: Category::Meta,
                        level: DiagnosticLevel::Deny,
                        message: format!("document {number}: {e}"),
                        location: Some(SourceLocation {
                            path: path.clone(),
                            line: 1,
                            column: 1,
                        }),
                        hint: None,
                        fatal: false,
                    });
                    continue;
                }
            };

            // Check for resource folder
            let resource_dir = source_path.join(number);
            let has_resources = resource_dir.is_dir();

            registry.entries.insert(
                number.to_string(),
                RegistryEntry {
                    document,
                    translations: HashMap::new(),
                    has_resources,
                },
            );
        }

        Ok(())
    }

    /// Discovers translations in `src/i18n/NNNN/<locale>.md`.
    fn discover_translations(
        source_path: &Path,
        registry: &mut Registry,
        diagnostics: &mut Diagnostics,
    ) -> Result<(), RegistryError> {
        let i18n_path = source_path.join("i18n");

        if !i18n_path.exists() {
            return Ok(());
        }

        let entries = std::fs::read_dir(&i18n_path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let dir_name = entry.file_name().to_string_lossy().to_string();

            // Skip non-four-digit directories
            if !Self::is_valid_document_number(&dir_name) {
                continue;
            }

            // Check if base document exists
            if !registry.entries.contains_key(&dir_name) {
                diagnostics.emit(Diagnostic {
                    slug: "orphan-translation".into(),
                    category: Category::I18n,
                    level: DiagnosticLevel::Deny,
                    message: format!("translation directory i18n/{dir_name}/ has no base document"),
                    location: Some(SourceLocation {
                        path: path.clone(),
                        line: 1,
                        column: 1,
                    }),
                    hint: Some(format!("create {dir_name}.md in the source directory")),
                    fatal: false,
                });
                continue;
            }

            // Discover translation files in this directory
            let translation_entries = std::fs::read_dir(&path)?;

            for translation_entry in translation_entries {
                let translation_entry = translation_entry?;
                let translation_path = translation_entry.path();

                if !translation_path.is_file() {
                    continue;
                }

                if !has_markdown_extension(&translation_path) {
                    continue;
                }

                // Extract locale from file stem (e.g., `ru_RU` from `ru_RU.md`)
                let locale = translation_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                if locale.is_empty() {
                    continue;
                }

                let content = std::fs::read_to_string(&translation_path)?;

                let translation = Translation {
                    locale: locale.clone(),
                    number: dir_name.clone(),
                    content,
                };

                if let Some(entry) = registry.entries.get_mut(&dir_name) {
                    entry.translations.insert(locale, translation);
                }
            }
        }

        Ok(())
    }

    /// Checks if a string is a valid four-digit document number.
    #[must_use]
    fn is_valid_document_number(s: &str) -> bool {
        s.len() == 4 && s.chars().all(|c| c.is_ascii_digit())
    }

    /// Returns the document number for a given number (resolving leading zeros).
    ///
    /// Per DR 003, leading zeros are permitted and ignored on resolution.
    #[must_use]
    pub fn resolve_number(&self, number: &str) -> Option<&str> {
        // Strip leading zeros and pad to four digits
        let stripped: String = number.chars().skip_while(|c| *c == '0').collect();
        if stripped.is_empty() {
            return None; // All zeros -> 0000, which is reserved
        }

        let padded = format!("{stripped:0>4}");

        if self.entries.contains_key(&padded) {
            self.entries
                .keys()
                .find(|k| *k == &padded)
                .map(String::as_str)
        } else {
            None
        }
    }
}
