use arxivlens::app::{App, Context};
use arxivlens::arxiv::{ArxivEntry, ArxivQueryResult};
use arxivlens::config::{Config, HighlightConfig};
use arxivlens::ui::{ArticleFeed, Theme};

fn create_test_articles() -> Vec<ArxivEntry> {
    vec![
        ArxivEntry::new(
            "Quantum Computing Fundamentals".to_string(),
            vec!["Alice Quantum".to_string()],
            "Research on quantum computing algorithms".to_string(),
            "quantum-001".to_string(),
            "2023-01-01".to_string(),
            "2023-01-01".to_string(),
        ),
        ArxivEntry::new(
            "Machine Learning Basics".to_string(),
            vec!["Bob ML".to_string()],
            "Deep learning and neural networks".to_string(),
            "ml-002".to_string(),
            "2023-01-02".to_string(),
            "2023-01-02".to_string(),
        ),
        ArxivEntry::new(
            "Real-Time Systems Design".to_string(),
            vec!["Charlie RT".to_string()],
            "Real-time operating systems and scheduling".to_string(),
            "rt-003".to_string(),
            "2023-01-03".to_string(),
            "2023-01-03".to_string(),
        ),
        ArxivEntry::new(
            "Distributed Computing".to_string(),
            vec!["Diana Dist".to_string()],
            "Systems programming with distributed architectures".to_string(),
            "dist-004".to_string(),
            "2023-01-04".to_string(),
            "2023-01-04".to_string(),
        ),
        ArxivEntry::new(
            "Network Protocols".to_string(),
            vec!["Eve Network".to_string()],
            "TCP/IP and network communication protocols".to_string(),
            "net-005".to_string(),
            "2023-01-05".to_string(),
            "2023-01-05".to_string(),
        ),
    ]
}

#[test]
fn test_search_integration_with_ui_feed() {
    // Create test data
    let articles = create_test_articles();
    let query_result = ArxivQueryResult {
        updated: "2023-01-01".to_string(),
        articles,
    };

    let highlight_config = HighlightConfig {
        authors: Some(vec!["Alice".to_string()]),
        keywords: Some(vec!["programming".to_string()]),
    };

    let theme = Theme::default();
    let config = Config::default();

    // Create app instance
    let mut app = App::new(&query_result, &highlight_config, theme, config);

    // Ensure search state is synchronized (should already be done in constructor, but let's be explicit)
    app.sync_search_state();

    // Simulate search by typing "Real" - should match "Real-Time Systems Design"
    for c in "Real".chars() {
        app.search_state.push_char(c);
    }

    // Verify search state has filtered results
    assert!(
        app.search_state.is_active(),
        "Search should be active after adding character"
    );

    let visible_count = app.get_visible_count();
    assert!(
        visible_count > 0,
        "Should have filtered results for 'Real' search"
    );

    // Simulate a render step by creating a fresh feed with the current search state
    let feed = ArticleFeed::new_with_search(
        &query_result,
        None, // No author highlighting for this test
        &app.theme,
        Some(&app.search_state),
    );

    // The feed length should match the number of filtered articles
    let expected_matches = app.search_state.filtered_count();
    assert_eq!(
        feed.len, expected_matches,
        "Feed length should match filtered article count"
    );
    assert_eq!(
        feed.len, visible_count,
        "Feed length should match visible count from app"
    );
    assert!(
        feed.len > 0,
        "Feed should not be empty when there are search matches"
    );
    assert!(
        feed.len < query_result.articles.len(),
        "Feed should be filtered (less than total articles)"
    );
}

#[test]
fn test_search_with_no_matches() {
    // Create test data
    let articles = create_test_articles();
    let query_result = ArxivQueryResult {
        updated: "2023-01-01".to_string(),
        articles,
    };

    let highlight_config = HighlightConfig {
        authors: None,
        keywords: None,
    };

    let theme = Theme::default();
    let config = Config::default();

    // Create app instance
    let mut app = App::new(&query_result, &highlight_config, theme, config);

    // Ensure search state is synchronized
    app.sync_search_state();

    // Search for something that won't match
    app.search_state.push_char('x');
    app.search_state.push_char('y');
    app.search_state.push_char('z');

    // Verify search state shows no results
    assert!(app.search_state.is_active(), "Search should be active");
    assert_eq!(
        app.search_state.filtered_count(),
        0,
        "Should have no filtered results"
    );

    // Simulate a render step by creating a fresh feed with the current search state
    let feed =
        ArticleFeed::new_with_search(&query_result, None, &app.theme, Some(&app.search_state));

    // Feed should be empty when no matches
    assert_eq!(feed.len, 0, "Feed should be empty when no search matches");
}

#[test]
fn test_search_state_initialization_in_app() {
    // Create test data
    let articles = create_test_articles();
    let query_result = ArxivQueryResult {
        updated: "2023-01-01".to_string(),
        articles,
    };

    let highlight_config = HighlightConfig {
        authors: None,
        keywords: None,
    };

    let theme = Theme::default();
    let config = Config::default();

    // Create app instance
    let mut app = App::new(&query_result, &highlight_config, theme, config);

    // Ensure search state is synchronized
    app.sync_search_state();

    // Initially search should not be active
    assert!(
        !app.search_state.is_active(),
        "Search should not be active initially"
    );

    // Set context to search - this should initialize the search state
    app.set_context(Context::Search);

    // After setting search context, search state should be initialized with articles
    assert_eq!(
        app.search_state.filtered_count(),
        query_result.articles.len(),
        "Search state should show all articles when no query is active"
    );
}

#[test]
fn test_search_exact_substring_priority() {
    // Create test data with predictable fixture
    let articles = create_test_articles();
    let query_result = ArxivQueryResult {
        updated: "2023-01-01".to_string(),
        articles,
    };

    let highlight_config = HighlightConfig {
        authors: None,
        keywords: None,
    };

    let theme = Theme::default();
    let config = Config::default();

    // Create app instance
    let mut app = App::new(&query_result, &highlight_config, theme, config);
    app.sync_search_state();

    // Enter search mode
    app.set_context(Context::Search);

    // Type "Real-Time" - should match exactly one result: "Real-Time Systems Design"
    for c in "Real-Time".chars() {
        app.handle_search_char(c);
    }

    // Verify filtered results
    let visible_articles = app.get_visible_articles();
    assert_eq!(
        visible_articles.len(),
        1,
        "Should match exactly 1 'Real-Time' article"
    );

    // Verify the correct article is matched
    let matched_title = &visible_articles[0].title;
    assert_eq!(
        matched_title, "Real-Time Systems Design",
        "Should match the Real-Time Systems Design article"
    );

    // Verify selection points to first (and only) filtered result
    assert_eq!(
        app.article_list_state.selected(),
        Some(0),
        "Selection should point to first filtered result"
    );

    // Check detail view sync - the selected article should match the search result
    if let Some(selected_idx) = app.article_list_state.selected() {
        if let Some(actual_idx) = app.get_actual_article_index(selected_idx) {
            let selected_article = &app.query_result.articles[actual_idx];
            assert_eq!(
                &selected_article.title, matched_title,
                "Detail view should show the matched search result"
            );
        } else {
            panic!("Should be able to get actual article index");
        }
    } else {
        panic!("Should have a selection");
    }
}

#[test]
fn test_search_selection_reset_on_query_change() {
    // Create test data
    let articles = create_test_articles();
    let query_result = ArxivQueryResult {
        updated: "2023-01-01".to_string(),
        articles,
    };

    let highlight_config = HighlightConfig {
        authors: None,
        keywords: None,
    };

    let theme = Theme::default();
    let config = Config::default();

    // Create app instance
    let mut app = App::new(&query_result, &highlight_config, theme, config);
    app.sync_search_state();

    // Enter search mode
    app.set_context(Context::Search);

    // First search for "Machine" - should match "Machine Learning Basics"
    for c in "Machine".chars() {
        app.handle_search_char(c);
    }

    // Verify we have results and selection is at first match
    assert!(app.get_visible_count() > 0, "Should have search results");
    assert_eq!(
        app.article_list_state.selected(),
        Some(0),
        "Selection should be at first match"
    );

    // Clear search and search for something with no matches
    app.search_state.clear();
    for c in "NonExistent".chars() {
        app.handle_search_char(c);
    }

    // Verify no results and no selection
    assert_eq!(app.get_visible_count(), 0, "Should have no search results");
    assert_eq!(
        app.article_list_state.selected(),
        None,
        "Selection should be None when no matches"
    );

    // Search for something that matches again
    app.search_state.clear();
    for c in "Network".chars() {
        app.handle_search_char(c);
    }

    // Verify selection resets to first match
    assert!(
        app.get_visible_count() > 0,
        "Should have search results again"
    );
    assert_eq!(
        app.article_list_state.selected(),
        Some(0),
        "Selection should reset to first match"
    );
}
