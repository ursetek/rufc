//! Configuration loading and validation (DR 008).

use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A linker template (DR 003, DR 008).
#[derive(Debug, Clone, Deserialize)]
pub struct Linker {
    /// Template for matching wikilink targets.
    /// Must contain exactly one `{}`.
    pub from: String,
    /// Template for generating URLs.
    /// Must contain exactly one `{}`.
    pub to: String,
}

/// Root configuration structure.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RufcConfig {
    /// Core configuration fields.
    pub general: GeneralConfig,
    /// Diagnostic level overrides (slug -> level).
    #[serde(default)]
    pub diagnostics: HashMap<String, String>,
}

/// The `[general]` section.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneralConfig {
    /// Source directory configuration.
    #[serde(default)]
    pub source: SourceConfig,
    /// Relationship modifier vocabulary (key -> (forward, reverse)).
    #[serde(default)]
    pub modifiers: HashMap<String, ModifierForms>,
    /// Index of the default linker in the `linkers` array.
    #[serde(default)]
    pub linker: usize,
    /// Array of linkers (required field).
    pub linkers: Vec<Linker>,
}

/// A pair of modifier forms (forward and reverse).
#[derive(Debug, Clone, Deserialize)]
pub struct ModifierForms(pub String, pub String);

/// The `source` subsection.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceConfig {
    /// Path to the document directory (relative to `Rufc.toml`).
    #[serde(default = "default_source_path")]
    pub path: String,
}

impl Default for SourceConfig {
    fn default() -> Self {
        Self {
            path: default_source_path(),
        }
    }
}

fn default_source_path() -> String {
    "src/".to_string()
}

/// Configuration errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// I/O error when reading configuration file.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// TOML syntax or structure parsing error.
    #[error("TOML parse error: {0}")]
    Parse(#[from] toml::de::Error),
    /// Required `general.linkers` field is missing.
    #[error("missing required field `general.linkers`")]
    MissingLinkers,
    /// Linker `from` template does not contain exactly one `{}`.
    #[error("linker[{0}]: `from` template must contain exactly one `{{}}`")]
    InvalidFromTemplate(usize),
    /// Linker `to` template does not contain exactly one `{}`.
    #[error("linker[{0}]: `to` template must contain exactly one `{{}}`")]
    InvalidToTemplate(usize),
    /// Two linkers have identical `from` templates.
    #[error("linkers[{0}] and linkers[{1}] have identical `from` templates")]
    DuplicateFromTemplate(usize, usize),
    /// Default linker index is out of bounds.
    #[error("`general.linker` index {0} is out of bounds (have {1} linkers)")]
    InvalidLinkerIndex(usize, usize),
    /// Modifier key contains non-alphanumeric characters.
    #[error("modifier key `{0}` contains non-alphanumeric characters")]
    InvalidModifierKey(String),
    /// Modifier has empty forward or reverse form.
    #[error("modifier `{0}` has empty forward or reverse form")]
    EmptyModifierForm(String),
    /// Diagnostic level value is invalid.
    #[error("diagnostics level `{0}` is invalid (must be `ignore`, `warn`, or `deny`)")]
    InvalidDiagnosticLevel(String),
}

impl RufcConfig {
    /// Loads configuration from a `Rufc.toml` file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read (I/O error)
    /// - The TOML syntax is invalid
    /// - The configuration fails validation
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: RufcConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Validates configuration after parsing.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required fields are missing
    /// - Linker templates are invalid
    /// - Duplicate `from` templates exist
    /// - Default linker index is out of bounds
    /// - Modifier keys or forms are invalid
    /// - Diagnostic level values are invalid
    pub fn validate(&self) -> Result<(), ConfigError> {
        // general.linkers is required
        if self.general.linkers.is_empty() {
            return Err(ConfigError::MissingLinkers);
        }

        // Validate linker templates
        for (i, linker) in self.general.linkers.iter().enumerate() {
            if linker.from.matches("{}").count() != 1 {
                return Err(ConfigError::InvalidFromTemplate(i));
            }
            if linker.to.matches("{}").count() != 1 {
                return Err(ConfigError::InvalidToTemplate(i));
            }
        }

        // Check for duplicate `from` templates
        let mut from_templates: HashMap<&str, usize> = HashMap::new();
        for (i, linker) in self.general.linkers.iter().enumerate() {
            if let Some(prev) = from_templates.insert(linker.from.as_str(), i) {
                return Err(ConfigError::DuplicateFromTemplate(prev, i));
            }
        }

        // Validate default linker index
        if self.general.linker >= self.general.linkers.len() {
            return Err(ConfigError::InvalidLinkerIndex(
                self.general.linker,
                self.general.linkers.len(),
            ));
        }

        // Validate modifier keys (ASCII alphanumeric, DR 002)
        for (key, forms) in &self.general.modifiers {
            if !key.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Err(ConfigError::InvalidModifierKey(key.clone()));
            }
            if forms.0.is_empty() || forms.1.is_empty() {
                return Err(ConfigError::EmptyModifierForm(key.clone()));
            }
        }

        // Validate diagnostic levels
        for level in self.diagnostics.values() {
            if level != "ignore" && level != "warn" && level != "deny" {
                return Err(ConfigError::InvalidDiagnosticLevel(level.clone()));
            }
        }

        Ok(())
    }

    /// Returns the absolute path to the document directory.
    #[must_use]
    pub fn source_path(&self, config_dir: &Path) -> PathBuf {
        config_dir.join(&self.general.source.path)
    }
}
