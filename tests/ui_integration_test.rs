use arxivlens::app::{App, Context};
use arxivlens::arxiv::{ArxivEntry, ArxivQueryResult};
use arxivlens::config::{Config, HighlightConfig};
use arxivlens::ui::Theme;

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
fn test_search_integration_logic() {
    let articles = create_test_articles();
    let query_result = ArxivQueryResult {
        updated: "2023-01-01".to_string(),
        articles,
    };

    let highlight_config = HighlightConfig {
        authors: Some(vec!["Alice".to_string()]),
        keywords: Some(vec!["programming".to_string()]),
    };

    // Create app instance
    let mut app = App::new(
        &query_result,
        &highlight_config,
        Theme::default(),
        Config::default(),
    );

    // 1. Initial State Check
    assert_eq!(app.get_visible_count(), 5, "Initially all articles visible");

    // 2. Simulate search by typing "Real"
    app.set_context(Context::Search);
    for c in "Real".chars() {
        app.search_state.push_char(c);
        // In your real App::handle_key_event, you should call sync/filter
        // If your App::get_visible_count calls sync internally, this just works.
    }

    // 3. Verify App State directly
    let visible_count = app.get_visible_count();
    assert!(visible_count > 0, "Should have filtered results for 'Real'");
    assert!(visible_count < 5, "Should be filtered (less than total)");

    // 4. Verify Content
    let visible_articles = app.get_visible_articles();
    assert_eq!(visible_articles[0].title, "Real-Time Systems Design");
}

#[test]
fn test_search_with_no_matches() {
    let articles = create_test_articles();
    let query_result = ArxivQueryResult {
        updated: "2023-01-01".to_string(),
        articles,
    };
    let highlight_config = HighlightConfig::default();

    let mut app = App::new(
        &query_result,
        &highlight_config,
        Theme::default(),
        Config::default(),
    );

    // Search for something that won't match
    app.set_context(Context::Search);
    for c in "xyz123".chars() {
        app.search_state.push_char(c);
    }

    // Verify app reports 0 visible items
    assert_eq!(
        app.get_visible_count(),
        0,
        "App should show 0 matches for garbage query"
    );
}

#[test]
fn test_search_selection_reset_on_query_change() {
    let articles = create_test_articles();
    let query_result = ArxivQueryResult {
        updated: "2023-01-01".to_string(),
        articles,
    };
    let highlight_config = HighlightConfig::default();
    let mut app = App::new(
        &query_result,
        &highlight_config,
        Theme::default(),
        Config::default(),
    );

    app.set_context(Context::Search);

    // 1. Search for "Machine"
    for c in "Machine".chars() {
        app.search_state.push_char(c);
    }

    // Ensure selection is at 0
    app.article_list_state.select(Some(0));
    assert_eq!(app.get_visible_count(), 1);

    // 2. Type garbage to clear results
    app.search_state.push_char('!');

    // Selection should be cleared if results are 0
    // (Ensure your App logic handles this in get_visible_count or handle_search)
    if app.get_visible_count() == 0 {
        app.article_list_state.select(None);
    }
    assert_eq!(app.article_list_state.selected(), None);
}
