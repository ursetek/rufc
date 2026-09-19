//! Configuration module tests.

use rufc::config::{ConfigError, RufcConfig};

#[test]
fn minimal_valid_config() {
    let toml_str = r#"
[general]
linkers = [
    { from = "RFC {}", to = "https://example.com/rfcs/{}" }
]
"#;
    let config: RufcConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.general.linkers.len(), 1);
    assert_eq!(config.general.source.path, "src/");
    assert_eq!(config.general.linker, 0);
}

#[test]
fn missing_linkers_is_parse_error() {
    let toml_str = r"
[general]
";
    let result: Result<RufcConfig, _> = toml::from_str(toml_str);
    assert!(result.is_err());
}

#[test]
fn invalid_from_template() {
    let toml_str = r#"
[general]
linkers = [
    { from = "RFC", to = "https://example.com/{}" }
]
"#;
    let config: RufcConfig = toml::from_str(toml_str).unwrap();
    let result = config.validate();
    assert!(matches!(result, Err(ConfigError::InvalidFromTemplate(0))));
}

#[test]
fn duplicate_from_templates() {
    let toml_str = r#"
[general]
linkers = [
    { from = "RFC {}", to = "https://example.com/{}" },
    { from = "RFC {}", to = "https://other.com/{}" }
]
"#;
    let config: RufcConfig = toml::from_str(toml_str).unwrap();
    let result = config.validate();
    assert!(matches!(
        result,
        Err(ConfigError::DuplicateFromTemplate(0, 1))
    ));
}

#[test]
fn unknown_field_in_general_rejected() {
    let toml_str = r#"
[general]
linkers = [{ from = "RFC {}", to = "https://example.com/{}" }]
unknown_field = "value"
"#;
    let result: Result<RufcConfig, _> = toml::from_str(toml_str);
    assert!(result.is_err());
}

#[test]
fn custom_source_path() {
    let toml_str = r#"
[general]
source.path = "docs/"
linkers = [{ from = "RFC {}", to = "https://example.com/{}" }]
"#;
    let config: RufcConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.general.source.path, "docs/");
}

#[test]
fn modifiers_with_valid_key() {
    let toml_str = r#"
[general]
linkers = [{ from = "RFC {}", to = "https://example.com/{}" }]

[general.modifiers]
SUPERSEDED = ["Supersedes", "Superseded by"]
"#;
    let config: RufcConfig = toml::from_str(toml_str).unwrap();
    assert!(config.validate().is_ok());
    assert!(config.general.modifiers.contains_key("SUPERSEDED"));
}

#[test]
fn modifier_key_with_non_alphanumeric() {
    let toml_str = r#"
[general]
linkers = [{ from = "RFC {}", to = "https://example.com/{}" }]

[general.modifiers]
"SUPER-SEDED" = ["Supersedes", "Superseded by"]
"#;
    let config: RufcConfig = toml::from_str(toml_str).unwrap();
    let result = config.validate();
    assert!(matches!(result, Err(ConfigError::InvalidModifierKey(_))));
}

#[test]
fn invalid_linker_index() {
    let toml_str = r#"
[general]
linker = 5
linkers = [{ from = "RFC {}", to = "https://example.com/{}" }]
"#;
    let config: RufcConfig = toml::from_str(toml_str).unwrap();
    let result = config.validate();
    assert!(matches!(result, Err(ConfigError::InvalidLinkerIndex(5, 1))));
}
