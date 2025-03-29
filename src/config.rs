use serde::Deserialize;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

const APP_DIR_NAME: &str = "arxivlens";
const CONFIG_FILE_NAME: &str = "config.toml";

const DEFAULT_ARXIV_CATEGORY: &str = "quant-ph";

#[derive(Debug)]
pub enum ConfigError {
    XdgError(String),
    IoError(std::io::Error),
    ParseError(toml::de::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::XdgError(e) => write!(f, "XDG directory error: {}", e),
            ConfigError::IoError(e) => write!(f, "Failed to read config file: {}", e),
            ConfigError::ParseError(e) => write!(f, "Failed to parse config file: {}", e),
        }
    }
}

impl Error for ConfigError {}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub query: QueryConfig,
    #[serde(default)]
    pub highlight: HighlightConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct QueryConfig {
    #[serde(default = "query_default_category")]
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct HighlightConfig {
    #[serde(default = "query_default_keywords")]
    pub keywords: Option<Vec<String>>,
    #[serde(default = "query_default_authors")]
    pub authors: Option<Vec<String>>,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            category: query_default_category(),
        }
    }
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            keywords: query_default_keywords(),
            authors: query_default_authors(),
        }
    }
}

fn query_default_category() -> String {
    DEFAULT_ARXIV_CATEGORY.to_string()
}
fn query_default_keywords() -> Option<Vec<String>> {
    None
}
fn query_default_authors() -> Option<Vec<String>> {
    None
}

impl Config {
    pub fn load() -> Result<Config, ConfigError> {
        let base_dirs = xdg::BaseDirectories::with_prefix(APP_DIR_NAME)
            .map_err(|e| ConfigError::XdgError(e.to_string()))?;

        let path = base_dirs.get_config_file(CONFIG_FILE_NAME);

        if path.exists() {
            Self::load_from_file(path)
        } else {
            Ok(Config::default())
        }
    }

    fn load_from_file(path: PathBuf) -> Result<Config, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(ConfigError::IoError)?;

        toml::from_str(&content).map_err(ConfigError::ParseError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_default_config() {
        let config = Config::default();
        assert_eq!(config.query.category, DEFAULT_ARXIV_CATEGORY);
        assert_eq!(config.highlight.keywords, None);
        assert_eq!(config.highlight.authors, None);
    }

    #[test]
    fn test_load_from_file() -> Result<(), ConfigError> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config_content = r#"
            [query]
            category = "cs.AI"
            
            [highlight]
            authors = ["Test Author"]
            keywords = ["quantum"]
        "#;
        fs::write(&config_path, config_content).unwrap();

        let config = Config::load_from_file(config_path)?;
        assert_eq!(config.query.category, "cs.AI");
        assert_eq!(
            config.highlight.authors,
            Some(vec!["Test Author".to_string()])
        );
        assert_eq!(config.highlight.keywords, Some(vec!["quantum".to_string()]));

        Ok(())
    }

    #[test]
    fn test_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let invalid_content = r#"
            [query
            category = "invalid"
        "#;
        fs::write(&config_path, invalid_content).unwrap();

        let result = Config::load_from_file(config_path);
        assert!(matches!(result, Err(ConfigError::ParseError(_))));
    }

    #[test]
    fn test_io_error() {
        let nonexistent_path = PathBuf::from("/nonexistent/path/config.toml");
        let result = Config::load_from_file(nonexistent_path);
        assert!(matches!(result, Err(ConfigError::IoError(_))));
    }

    #[test]
    fn test_partial_config() -> Result<(), ConfigError> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        // Only specify query section, highlight should use defaults
        let config_content = r#"
            [query]
            category = "math.AG"
        "#;
        fs::write(&config_path, config_content).unwrap();

        let config = Config::load_from_file(config_path)?;
        assert_eq!(config.query.category, "math.AG");
        assert_eq!(config.highlight.keywords, None);
        assert_eq!(config.highlight.authors, None);

        Ok(())
    }

    #[test]
    fn test_empty_config() -> Result<(), ConfigError> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "").unwrap();

        let config = Config::load_from_file(config_path)?;
        // Should use all defaults
        assert_eq!(config.query.category, DEFAULT_ARXIV_CATEGORY);
        assert_eq!(config.highlight.keywords, None);
        assert_eq!(config.highlight.authors, None);

        Ok(())
    }

    #[test]
    fn test_multiple_authors_and_keywords() -> Result<(), ConfigError> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config_content = r#"
            [query]
            category = "quant-ph"
            
            [highlight]
            authors = ["Einstein", "Bohr", "Heisenberg"]
            keywords = ["quantum", "entanglement", "superposition"]
        "#;
        fs::write(&config_path, config_content).unwrap();

        let config = Config::load_from_file(config_path)?;
        assert_eq!(
            config.highlight.authors,
            Some(vec![
                "Einstein".to_string(),
                "Bohr".to_string(),
                "Heisenberg".to_string()
            ])
        );
        assert_eq!(
            config.highlight.keywords,
            Some(vec![
                "quantum".to_string(),
                "entanglement".to_string(),
                "superposition".to_string()
            ])
        );

        Ok(())
    }

    #[test]
    fn test_malformed_category() -> Result<(), ConfigError> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config_content = r#"
            [query]
            category = 123  # Should be a string, not a number

            [highlight]
            authors = ["Test Author"]
        "#;
        fs::write(&config_path, config_content).unwrap();

        let result = Config::load_from_file(config_path);
        assert!(matches!(result, Err(ConfigError::ParseError(_))));
        Ok(())
    }

    #[test]
    fn test_case_sensitivity() -> Result<(), ConfigError> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config_content = r#"
            [query]
            category = "QUANT-PH"  # Different case from default
            
            [highlight]
            authors = ["EINSTEIN", "Bohr", "heisenberg"]
            keywords = ["QUANTUM", "Entanglement", "superposition"]
        "#;
        fs::write(&config_path, config_content).unwrap();

        let config = Config::load_from_file(config_path)?;
        assert_eq!(config.query.category, "QUANT-PH"); // Category should preserve case
        assert_eq!(
            config.highlight.authors,
            Some(vec![
                "EINSTEIN".to_string(),
                "Bohr".to_string(),
                "heisenberg".to_string()
            ])
        );
        assert_eq!(
            config.highlight.keywords,
            Some(vec![
                "QUANTUM".to_string(),
                "Entanglement".to_string(),
                "superposition".to_string()
            ])
        );

        Ok(())
    }

    #[test]
    fn test_invalid_highlight_format() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config_content = r#"
            [query]
            category = "quant-ph"
            
            [highlight]
            authors = "Einstein"  # Should be an array, not a string
            keywords = ["quantum"]
        "#;
        fs::write(&config_path, config_content).unwrap();

        let result = Config::load_from_file(config_path);
        assert!(matches!(result, Err(ConfigError::ParseError(_))));
    }

    #[test]
    fn test_duplicate_sections() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config_content = r#"
            [query]
            category = "quant-ph"
            
            [highlight]
            authors = ["Einstein"]
            
            [query]  # Duplicate section
            category = "cs.AI"
        "#;
        fs::write(&config_path, config_content).unwrap();

        let result = Config::load_from_file(config_path);
        assert!(matches!(result, Err(ConfigError::ParseError(_))));
    }

    #[test]
    fn test_xdg_error_handling() {
        // This test might be environment-dependent
        // We can at least verify that the XdgError variant exists and can be created
        let error = ConfigError::XdgError("test error".to_string());
        assert!(matches!(error, ConfigError::XdgError(_)));

        // Test error formatting
        let error_str = format!("{}", error);
        assert_eq!(error_str, "XDG directory error: test error");
    }
}
