#[cfg(test)]
mod golden_tests {
    use arxivlens::golden_test;
    use arxivlens::ui::components::{
        ArticleListComponent, ConfigPopupComponent, SearchBarComponent,
    };
    use arxivlens::ui::testing::GoldenTester;
    use std::path::PathBuf;

    #[test]
    fn test_golden_tester_setup() {
        let test_dir = PathBuf::from("tests/golden");
        let tester = GoldenTester::new(&test_dir);

        // This test just verifies the tester can be created
        // Actual component tests would be added here once the component system is fully integrated
        assert_eq!(tester.test_dir, test_dir);
    }

    golden_test!(SearchBarComponent, test_search_bar_golden);
    golden_test!(ArticleListComponent, test_article_list_golden);
    golden_test!(ConfigPopupComponent, test_config_popup_golden);
}
