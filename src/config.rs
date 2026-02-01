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
            Self::XdgError(e) => write!(f, "XDG directory error: {e}"),
            Self::IoError(e) => write!(f, "Failed to read config file: {e}"),
            Self::ParseError(e) => write!(f, "Failed to parse config file: {e}"),
        }
    }
}

impl Error for ConfigError {}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub query: QueryConfig,
    #[serde(default)]
    pub pinned: PinnedConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct QueryConfig {
    #[serde(default = "query_default_category")]
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
pub struct PinnedConfig {
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "storage_default_database_name")]
    pub database_name: String,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            category: query_default_category(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database_name: storage_default_database_name(),
        }
    }
}

fn query_default_category() -> String {
    DEFAULT_ARXIV_CATEGORY.to_string()
}

fn storage_default_database_name() -> String {
    "starred.db".to_string()
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let base_dirs = xdg::BaseDirectories::with_prefix(APP_DIR_NAME)
            .map_err(|e| ConfigError::XdgError(e.to_string()))?;

        let path = base_dirs.get_config_file(CONFIG_FILE_NAME);

        if path.exists() {
            Self::load_from_file(path)
        } else {
            Ok(Self::default())
        }
    }

    fn load_from_file(path: PathBuf) -> Result<Self, ConfigError> {
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
        assert_eq!(config.pinned.authors, Vec::<String>::new());
        assert_eq!(config.pinned.categories, Vec::<String>::new());
        assert_eq!(config.storage.database_name, "starred.db");
    }

    #[test]
    fn test_load_from_file() -> Result<(), ConfigError> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config_content = r#"
            [query]
            category = "cs.AI"
            
            [pinned]
            authors = ["Test Author"]
            categories = ["quant-ph", "cs.AI"]
            
            [storage]
            database_name = "custom.db"
        "#;
        fs::write(&config_path, config_content).unwrap();

        let config = Config::load_from_file(config_path)?;
        assert_eq!(config.query.category, "cs.AI");
        assert_eq!(config.pinned.authors, vec!["Test Author".to_string()]);
        assert_eq!(
            config.pinned.categories,
            vec!["quant-ph".to_string(), "cs.AI".to_string()]
        );
        assert_eq!(config.storage.database_name, "custom.db");

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
        // Only specify query section, pinned and storage should use defaults
        let config_content = r#"
            [query]
            category = "math.AG"
        "#;
        fs::write(&config_path, config_content).unwrap();

        let config = Config::load_from_file(config_path)?;
        assert_eq!(config.query.category, "math.AG");
        assert_eq!(config.pinned.authors, Vec::<String>::new());
        assert_eq!(config.pinned.categories, Vec::<String>::new());
        assert_eq!(config.storage.database_name, "starred.db");

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
        assert_eq!(config.pinned.authors, Vec::<String>::new());
        assert_eq!(config.pinned.categories, Vec::<String>::new());
        assert_eq!(config.storage.database_name, "starred.db");

        Ok(())
    }

    #[test]
    fn test_multiple_authors_and_categories() -> Result<(), ConfigError> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config_content = r#"
            [query]
            category = "quant-ph"
            
            [pinned]
            authors = ["Einstein", "Bohr", "Heisenberg"]
            categories = ["quant-ph", "cond-mat", "hep-th"]
            
            [storage]
            database_name = "physics.db"
        "#;
        fs::write(&config_path, config_content).unwrap();

        let config = Config::load_from_file(config_path)?;
        assert_eq!(
            config.pinned.authors,
            vec![
                "Einstein".to_string(),
                "Bohr".to_string(),
                "Heisenberg".to_string()
            ]
        );
        assert_eq!(
            config.pinned.categories,
            vec![
                "quant-ph".to_string(),
                "cond-mat".to_string(),
                "hep-th".to_string()
            ]
        );
        assert_eq!(config.storage.database_name, "physics.db");

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
            
            [pinned]
            authors = ["EINSTEIN", "Bohr", "heisenberg"]
            categories = ["QUANT-PH", "Cond-Mat", "hep-th"]
        "#;
        fs::write(&config_path, config_content).unwrap();

        let config = Config::load_from_file(config_path)?;
        assert_eq!(config.query.category, "QUANT-PH"); // Category should preserve case
        assert_eq!(
            config.pinned.authors,
            vec![
                "EINSTEIN".to_string(),
                "Bohr".to_string(),
                "heisenberg".to_string()
            ]
        );
        assert_eq!(
            config.pinned.categories,
            vec![
                "QUANT-PH".to_string(),
                "Cond-Mat".to_string(),
                "hep-th".to_string()
            ]
        );

        Ok(())
    }

    #[test]
    fn test_invalid_pinned_format() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config_content = r#"
            [query]
            category = "quant-ph"
            
            [pinned]
            authors = "Einstein"  # Should be an array, not a string
            categories = ["quant-ph"]
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
            
            [pinned]
            authors = ["Einstein"]
            
            [query]  # Duplicate section
            category = "cs.AI"
        "#;
        fs::write(&config_path, config_content).unwrap();

        let result = Config::load_from_file(config_path);
        assert!(matches!(result, Err(ConfigError::ParseError(_))));
    }

    #[test]
    fn test_storage_config_defaults() -> Result<(), ConfigError> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config_content = r#"
            [query]
            category = "quant-ph"
            
            [pinned]
            authors = ["Einstein"]
            categories = ["quant-ph"]
            # No storage section - should use defaults
        "#;
        fs::write(&config_path, config_content).unwrap();

        let config = Config::load_from_file(config_path)?;
        assert_eq!(config.storage.database_name, "starred.db");

        Ok(())
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
