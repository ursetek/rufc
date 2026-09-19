//! Diagnostics system (DR 009).
//!
//! This module is orthogonal to the rest of the library: every other
//! module depends on it, but it depends on none of them (DR 001).

use std::path::PathBuf;

/// Diagnostic category (DR 009, section 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    /// `Rufc.toml` parsing and validation errors.
    Config,
    /// `toml,meta` block parsing and validation errors.
    Meta,
    /// Wikilink resolution and cross-reference validation errors.
    Links,
    /// Translation processing and locale handling errors.
    I18n,
    /// Source directory structure and resource file errors.
    Structure,
    /// Filesystem and I/O errors.
    Io,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Config => "config",
            Self::Meta => "meta",
            Self::Links => "links",
            Self::I18n => "i18n",
            Self::Structure => "structure",
            Self::Io => "io",
        };
        write!(f, "{s}")
    }
}

/// Diagnostic level (DR 009, section 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    /// Condition is not reported; build continues.
    Ignore,
    /// Condition is reported but does not affect the build outcome.
    Warn,
    /// Condition is reported as an error.
    Deny,
}

impl DiagnosticLevel {
    /// Parses a level from a string (used for `Rufc.toml`).
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "ignore" => Some(Self::Ignore),
            "warn" => Some(Self::Warn),
            "deny" => Some(Self::Deny),
            _ => None,
        }
    }
}

/// Source location in a file.
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// File path.
    pub path: PathBuf,
    /// Line number (1-based).
    pub line: usize,
    /// Column number (1-based).
    pub column: usize,
}

/// A single diagnostic condition.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Unique identifier for the diagnostic type.
    pub slug: String,
    /// Category describing what part of the system reported it.
    pub category: Category,
    /// Severity level determining how the condition is handled.
    pub level: DiagnosticLevel,
    /// Human-readable description of the condition.
    pub message: String,
    /// Optional source location where the condition was detected.
    pub location: Option<SourceLocation>,
    /// Optional hint for fixing the condition.
    pub hint: Option<String>,
    /// When `true`, requires immediate build termination.
    pub fatal: bool,
}

/// Accumulator for diagnostics.
#[derive(Debug, Default)]
pub struct Diagnostics {
    items: Vec<Diagnostic>,
}

impl Diagnostics {
    /// Creates a new empty diagnostics accumulator.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Emits a diagnostic.
    ///
    /// When `fatal = true`, all previously accumulated diagnostics are
    /// discarded; only this one remains (immediate termination, DR 009
    /// section 5).
    pub fn emit(&mut self, diag: Diagnostic) {
        if diag.level == DiagnosticLevel::Ignore {
            return;
        }
        if diag.fatal {
            self.items.clear();
            self.items.push(diag);
        } else {
            self.items.push(diag);
        }
    }

    /// Returns `true` if at least one diagnostic has level `Deny`.
    #[must_use]
    pub fn has_denied(&self) -> bool {
        self.items.iter().any(|d| d.level == DiagnosticLevel::Deny)
    }

    /// Returns a slice of all accumulated diagnostics.
    #[must_use]
    pub fn items(&self) -> &[Diagnostic] {
        &self.items
    }

    /// Renders all diagnostics to a string using `annotate-snippets`.
    #[must_use]
    pub fn render(&self, use_ansi: bool) -> String {
        use annotate_snippets::{Annotation, Group, Level, Renderer, Snippet};

        let renderer = if use_ansi {
            Renderer::styled()
        } else {
            Renderer::plain()
        };

        let mut output = String::new();

        for diag in &self.items {
            let level = match diag.level {
                DiagnosticLevel::Deny => Level::ERROR,
                DiagnosticLevel::Warn => Level::WARNING,
                DiagnosticLevel::Ignore => continue,
            };

            let title_str = format!("{} ({})", diag.message, diag.slug);
            let mut group = Group::with_title(level.primary_title(&title_str));

            if let Some(loc) = &diag.location {
                let path_string = loc.path.to_string_lossy().into_owned();
                let snippet: Snippet<'_, Annotation<'_>> =
                    Snippet::source("").path(path_string).line_start(loc.line);
                group = group.element(snippet);
            }

            if let Some(hint) = &diag.hint {
                group = group.element(Level::NOTE.message(hint));
            }

            output.push_str(&renderer.render(&[group]));
            output.push('\n');
        }

        output
    }
}
