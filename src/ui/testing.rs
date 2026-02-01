//! Golden file testing utilities for UI components.
//!
//! This module provides snapshot testing capabilities to prevent UI regressions.
//! To update golden files when UI changes are intentional, run:
//! `UPDATE_GOLDEN=1 cargo test --test ui_golden_tests`

use ratatui::{backend::TestBackend, Terminal};
use std::fs;
use std::path::{Path, PathBuf};

use crate::ui::component::TestableComponent;
use crate::ui::theme::Theme;

/// Golden file testing utilities for UI components
pub struct GoldenTester {
    pub test_dir: PathBuf,
    update_golden: bool,
}

impl GoldenTester {
    pub fn new<P: AsRef<Path>>(test_dir: P) -> Self {
        Self {
            test_dir: test_dir.as_ref().to_path_buf(),
            update_golden: std::env::var("UPDATE_GOLDEN").is_ok(),
        }
    }

    /// Test a component against its golden file
    pub fn test_component<C>(
        &self,
        component: &mut C,
        width: u16,
        height: u16,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        C: for<'a> TestableComponent<'a>,
    {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend)?;
        let theme = Theme::default();

        let output = terminal.draw(|frame| {
            let area = frame.size();
            let mut state = C::get_test_state();
            component.render(frame, area, &mut state, &theme);
        })?;

        let rendered = format!("{:?}", output);
        let golden_path = self.test_dir.join(format!("{}.golden", C::test_name()));

        if self.update_golden {
            fs::create_dir_all(&self.test_dir)?;
            fs::write(&golden_path, &rendered)?;
            println!("Updated golden file: {}", golden_path.display());
        } else {
            let expected = fs::read_to_string(&golden_path).map_err(|_| {
                format!(
                    "Golden file not found: {}. Run with UPDATE_GOLDEN=1 to create it.",
                    golden_path.display()
                )
            })?;

            if rendered != expected {
                return Err(format!(
                    "Golden file mismatch for {}\nExpected:\n{}\nActual:\n{}",
                    C::test_name(),
                    expected,
                    rendered
                )
                .into());
            }
        }

        Ok(())
    }

    /// Test multiple sizes for responsive design
    pub fn test_component_responsive<C>(
        &self,
        component: &mut C,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        C: for<'a> TestableComponent<'a>,
    {
        let sizes = vec![
            (80, 24),  // Standard terminal
            (120, 30), // Large terminal
            (40, 12),  // Small terminal
            (20, 6),   // Tiny terminal
        ];

        for (width, height) in sizes {
            let backend = TestBackend::new(width, height);
            let mut terminal = Terminal::new(backend)?;
            let theme = Theme::default();

            let output = terminal.draw(|frame| {
                let area = frame.size();
                let mut state = C::get_test_state();
                component.render(frame, area, &mut state, &theme);
            })?;

            let rendered = format!("{:?}", output);
            let golden_path =
                self.test_dir
                    .join(format!("{}_{}x{}.golden", C::test_name(), width, height));

            if self.update_golden {
                fs::create_dir_all(&self.test_dir)?;
                fs::write(&golden_path, &rendered)?;
            } else {
                let expected = fs::read_to_string(&golden_path).map_err(|_| {
                    format!(
                        "Golden file not found: {}. Run with UPDATE_GOLDEN=1 to create it.",
                        golden_path.display()
                    )
                })?;

                if rendered != expected {
                    return Err(format!(
                        "Golden file mismatch for {} at {}x{}\nExpected:\n{}\nActual:\n{}",
                        C::test_name(),
                        width,
                        height,
                        expected,
                        rendered
                    )
                    .into());
                }
            }
        }

        Ok(())
    }
}

/// Macro to simplify golden file testing
#[macro_export]
macro_rules! golden_test {
    ($component:ty, $test_name:ident) => {
        #[test]
        fn $test_name() {
            use $crate::ui::component::TestableComponent;
            use $crate::ui::testing::GoldenTester;

            let tester = GoldenTester::new("tests/golden");
            let mut component = <$component>::create_test_instance();

            tester
                .test_component(&mut component, 80, 24)
                .expect("Golden file test failed");
        }
    };

    ($component:ty, $test_name:ident, responsive) => {
        #[test]
        fn $test_name() {
            use $crate::ui::component::TestableComponent;
            use $crate::ui::testing::GoldenTester;

            let tester = GoldenTester::new("tests/golden");
            let mut component = <$component>::create_test_instance();

            tester
                .test_component_responsive(&mut component)
                .expect("Responsive golden file test failed");
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_golden_tester_creation() {
        let temp_dir = TempDir::new().unwrap();
        let tester = GoldenTester::new(temp_dir.path());
        assert_eq!(tester.test_dir, temp_dir.path());
    }

    #[test]
    fn test_update_golden_env_var() {
        std::env::set_var("UPDATE_GOLDEN", "1");
        let temp_dir = TempDir::new().unwrap();
        let tester = GoldenTester::new(temp_dir.path());
        assert!(tester.update_golden);
        std::env::remove_var("UPDATE_GOLDEN");
    }
}
