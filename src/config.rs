use serde::Deserialize;

const APP_DIR_NAME: &str = "arxivlens";
const CONFIG_FILE_NAME: &str = "config.toml";

const DEFAULT_ARXIV_CATEGORY: &str = "quant-ph";

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
    pub fn load() -> Config {
        let path = xdg::BaseDirectories::with_prefix(APP_DIR_NAME)
            .unwrap()
            .get_config_file(CONFIG_FILE_NAME);
        if path.exists() {
            let content = std::fs::read_to_string(path).unwrap();
            toml::from_str(&content).unwrap()
        } else {
            Config::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let actual = Config::default();
        let expected = Config {
            query: QueryConfig {
                category: "quant-ph".into(),
            },
            highlight: HighlightConfig {
                keywords: None,
                authors: None,
            },
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_config_complete_toml() {
        let toml = r#"
            [query]
            category = "quant-ph"
            [highlight]
            keywords = ["apple", "berry"]
            authors = ["Schrodinger", "Becquerel"]
        "#;
        let actual: Config = toml::from_str(toml).unwrap();
        let expected = Config {
            query: QueryConfig {
                category: "quant-ph".into(),
            },
            highlight: HighlightConfig {
                keywords: Some(vec!["apple".to_string(), "berry".to_string()]),
                authors: Some(vec!["Schrodinger".to_string(), "Becquerel".to_string()]),
            },
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_config_partial_toml() {
        let toml = r#"
            [highlight]
            authors = ["Schrodinger", "Becquerel"]
        "#;
        let actual: Config = toml::from_str(toml).unwrap();
        let expected = Config {
            query: QueryConfig {
                category: "quant-ph".into(),
            },
            highlight: HighlightConfig {
                keywords: None,
                authors: Some(vec!["Schrodinger".to_string(), "Becquerel".to_string()]),
            },
        };
        assert_eq!(actual, expected);
    }
}
