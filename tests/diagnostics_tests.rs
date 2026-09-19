//! Diagnostics module tests.

use rufc::diagnostics::{Category, Diagnostic, DiagnosticLevel, Diagnostics};

#[test]
fn has_denied_returns_false_when_empty() {
    let diag = Diagnostics::new();
    assert!(!diag.has_denied());
}

#[test]
fn warn_does_not_trigger_denied() {
    let mut diag = Diagnostics::new();
    diag.emit(Diagnostic {
        slug: "test-warning".into(),
        category: Category::Config,
        level: DiagnosticLevel::Warn,
        message: "test warning".into(),
        location: None,
        hint: None,
        fatal: false,
    });
    assert!(!diag.has_denied());
}

#[test]
fn deny_triggers_has_denied() {
    let mut diag = Diagnostics::new();
    diag.emit(Diagnostic {
        slug: "test-error".into(),
        category: Category::Config,
        level: DiagnosticLevel::Deny,
        message: "test error".into(),
        location: None,
        hint: None,
        fatal: false,
    });
    assert!(diag.has_denied());
}

#[test]
fn ignore_level_is_not_accumulated() {
    let mut diag = Diagnostics::new();
    diag.emit(Diagnostic {
        slug: "ignored".into(),
        category: Category::Config,
        level: DiagnosticLevel::Ignore,
        message: "should be ignored".into(),
        location: None,
        hint: None,
        fatal: false,
    });
    assert_eq!(diag.items().len(), 0);
}

#[test]
fn fatal_clears_previous_diagnostics() {
    let mut diag = Diagnostics::new();
    diag.emit(Diagnostic {
        slug: "warning-1".into(),
        category: Category::Config,
        level: DiagnosticLevel::Warn,
        message: "first".into(),
        location: None,
        hint: None,
        fatal: false,
    });
    diag.emit(Diagnostic {
        slug: "warning-2".into(),
        category: Category::Config,
        level: DiagnosticLevel::Warn,
        message: "second".into(),
        location: None,
        hint: None,
        fatal: false,
    });
    assert_eq!(diag.items().len(), 2);

    diag.emit(Diagnostic {
        slug: "fatal-error".into(),
        category: Category::Io,
        level: DiagnosticLevel::Deny,
        message: "fatal".into(),
        location: None,
        hint: None,
        fatal: true,
    });
    assert_eq!(diag.items().len(), 1);
    assert_eq!(diag.items()[0].slug, "fatal-error");
}

#[test]
fn render_plain_contains_slug_and_message() {
    let mut diag = Diagnostics::new();
    diag.emit(Diagnostic {
        slug: "test".into(),
        category: Category::Config,
        level: DiagnosticLevel::Deny,
        message: "test error".into(),
        location: None,
        hint: None,
        fatal: false,
    });
    let output = diag.render(false);
    assert!(output.contains("test error"));
    assert!(output.contains("test"));
}

#[test]
fn diagnostic_level_parse() {
    assert_eq!(
        DiagnosticLevel::parse("ignore"),
        Some(DiagnosticLevel::Ignore)
    );
    assert_eq!(DiagnosticLevel::parse("warn"), Some(DiagnosticLevel::Warn));
    assert_eq!(DiagnosticLevel::parse("deny"), Some(DiagnosticLevel::Deny));
    assert_eq!(DiagnosticLevel::parse("invalid"), None);
}
