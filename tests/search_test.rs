use arxivlens::app::search::SearchState;
use arxivlens::arxiv::{ArxivEntry, ArxivQueryResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct SyntheticArticle {
    title: String,
    authors: Vec<String>,
    summary: String,
    id: String,
    updated: String,
    published: String,
}

impl From<SyntheticArticle> for ArxivEntry {
    fn from(article: SyntheticArticle) -> Self {
        ArxivEntry::new(
            article.title,
            article.authors,
            article.summary,
            article.id,
            article.updated,
            article.published,
        )
    }
}

/// Load synthetic test dataset from JSON fixture and optionally fill to requested size
fn generate_test_dataset(size: usize) -> Vec<ArxivEntry> {
    // Load base articles from JSON fixture
    const SYNTHETIC_JSON: &str = include_str!("fixtures/synthetic_articles.json");
    let synthetic_articles: Vec<SyntheticArticle> =
        serde_json::from_str(SYNTHETIC_JSON).expect("Failed to parse synthetic articles JSON");

    let mut articles: Vec<ArxivEntry> = synthetic_articles
        .into_iter()
        .map(ArxivEntry::from)
        .collect();

    // If we need more articles than in the fixture, fill with noise
    if size > articles.len() {
        let mut id_counter = articles.len() + 1;

        let noise_prefixes = [
            "Analysis of",
            "Study on",
            "Investigation into",
            "Research on",
            "Exploration of",
            "Survey of",
            "Review of",
            "Advances in",
            "Novel Approaches to",
            "Theoretical Framework for",
        ];

        let noise_topics = vec![
            "distributed systems",
            "network protocols",
            "database optimization",
            "compiler design",
            "operating systems",
            "computer graphics",
            "human-computer interaction",
            "software engineering",
            "cybersecurity",
            "data structures",
            "algorithm complexity",
            "parallel computing",
            "web technologies",
            "mobile development",
            "cloud computing",
            "blockchain technology",
            "artificial intelligence",
            "robotics",
            "bioinformatics",
            "computational biology",
        ];

        let noise_authors = vec![
            "John Doe",
            "Jane Smith",
            "Alex Johnson",
            "Sarah Wilson",
            "Michael Brown",
            "Emily Davis",
            "David Miller",
            "Lisa Garcia",
            "Robert Martinez",
            "Jennifer Lopez",
            "Christopher Lee",
            "Amanda Taylor",
            "Daniel Anderson",
            "Michelle Thomas",
            "James Jackson",
        ];

        // Fill remaining slots with random noise articles
        while articles.len() < size {
            let prefix = &noise_prefixes[id_counter % noise_prefixes.len()];
            let topic = &noise_topics[id_counter % noise_topics.len()];
            let author = &noise_authors[id_counter % noise_authors.len()];

            let title = format!("{} {}", prefix, topic);
            let summary = format!("This paper presents {} in the context of {}. We propose novel methods and evaluate their effectiveness through comprehensive experiments.", prefix.to_lowercase(), topic);

            articles.push(ArxivEntry::new(
                title,
                vec![author.to_string()],
                summary,
                format!("cs.DC/{:04}", id_counter),
                "2024-01-05".to_string(),
                "2024-01-05".to_string(),
            ));
            id_counter += 1;
        }
    } else {
        // If requested size is smaller, truncate
        articles.truncate(size);
    }

    articles
}

fn load_fixture(name: &str) -> String {
    let mut fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fixture_path.push("tests");
    fixture_path.push("fixtures");
    fixture_path.push(name);

    fs::read_to_string(&fixture_path)
        .unwrap_or_else(|_| panic!("Failed to read fixture file: {}", fixture_path.display()))
}

fn load_arxiv_fixture() -> Vec<ArxivEntry> {
    let fixture_content = load_fixture("sample_arxiv.xml");
    let query_result =
        ArxivQueryResult::from_xml_content(&fixture_content).expect("Failed to parse fixture XML");
    query_result.articles
}

/// Helper function to initialize state with articles
fn setup_search(articles: &[ArxivEntry]) -> SearchState {
    let mut state = SearchState::new();
    state.set_articles(articles);
    state
}

#[test]
fn test_fixture_case_insensitive_search() {
    let articles = load_arxiv_fixture();
    let mut search_state = setup_search(&articles);

    assert!(!articles.is_empty(), "Fixture should contain articles");

    // Test case insensitive matching - Notice &articles is removed
    search_state.update_query("QUANTUM".to_string());
    let uppercase_matches = search_state.filtered_count();

    search_state.update_query("quantum".to_string());
    let lowercase_matches = search_state.filtered_count();

    search_state.update_query("Quantum".to_string());
    let mixed_case_matches = search_state.filtered_count();

    assert_eq!(uppercase_matches, lowercase_matches);
    assert_eq!(lowercase_matches, mixed_case_matches);
}

#[test]
fn test_fixture_empty_query() {
    let articles = load_arxiv_fixture();
    let mut search_state = setup_search(&articles);

    search_state.update_query("".to_string());

    assert_eq!(search_state.filtered_count(), articles.len());
    let expected_indices: Vec<usize> = (0..articles.len()).collect();
    assert_eq!(search_state.filtered_indices, expected_indices);
}

#[test]
fn test_fixture_no_matches() {
    let articles = load_arxiv_fixture();
    let mut search_state = setup_search(&articles);

    search_state.update_query("xyzabc123nonexistent".to_string());

    assert_eq!(search_state.filtered_count(), 0);
    assert!(search_state.filtered_indices.is_empty());
}

#[test]
fn test_synthetic_dataset_quantum_search() {
    let articles = generate_test_dataset(100);
    let mut search_state = setup_search(&articles);

    search_state.update_query("quantum".to_string());

    search_state
        .verify_indices_integrity(&articles)
        .expect("All filtered indices should be valid");

    assert!(search_state.filtered_count() >= 6);

    let rendered_titles = search_state.get_rendered_titles(&articles);
    assert_eq!(rendered_titles.len(), search_state.filtered_indices.len());
}

#[test]
fn test_synthetic_dataset_machine_learning_search() {
    let articles = generate_test_dataset(50);
    let mut search_state = setup_search(&articles);

    search_state.update_query("machine learning".to_string());

    search_state
        .verify_indices_integrity(&articles)
        .expect("All filtered indices should be valid");

    assert!(search_state.filtered_count() >= 6);
}

#[test]
fn test_synthetic_dataset_rust_ranking() {
    let articles = generate_test_dataset(50);
    let mut search_state = setup_search(&articles);

    search_state.update_query("rust programming".to_string());

    assert!(search_state.filtered_count() > 0);

    if search_state.filtered_count() >= 3 {
        let top_3_titles: Vec<String> = search_state.get_rendered_titles(&articles)[..3].to_vec();
        let has_exact_match = top_3_titles
            .iter()
            .any(|title| title.to_lowercase().contains("rust programming"));
        assert!(has_exact_match);
    }
}

#[test]
fn test_synthetic_dataset_unicode_handling() {
    let articles = generate_test_dataset(50);
    let mut search_state = setup_search(&articles);

    search_state.update_query("量子".to_string());

    search_state
        .verify_indices_integrity(&articles)
        .expect("All filtered indices should be valid");

    assert!(search_state.filtered_count() > 0);
}

#[test]
fn test_synthetic_dataset_large_scale() {
    let articles = generate_test_dataset(1000);
    let mut search_state = setup_search(&articles);

    let start_time = std::time::Instant::now();
    search_state.update_query("analysis".to_string());
    let duration = start_time.elapsed();

    assert!(duration.as_secs() < 1);
    assert!(search_state.filtered_count() > 0);
}

#[test]
fn test_keyword_filtering() {
    let articles = generate_test_dataset(100);
    let mut search_state = setup_search(&articles);

    search_state.update_query("machine".to_string());
    let relevance_info = search_state.get_match_relevance(&articles);

    for (_, is_relevant) in &relevance_info {
        assert!(*is_relevant);
    }

    assert!(search_state.filtered_count() > 0);
    assert!(search_state.filtered_count() < 50);
}
