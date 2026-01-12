#![forbid(unsafe_code)]

//! Human-readable output formatter with colorization support

use crate::engine::aggregator::{AggregationResult, RuleRegionStatus};
use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Human-readable output formatter
///
/// Formats aggregation results for terminal display with optional colors.
pub struct HumanFormatter {
    color_choice: ColorChoice,
}

impl HumanFormatter {
    /// Creates a new HumanFormatter with the specified color choice
    pub fn new(color_choice: ColorChoice) -> Self {
        HumanFormatter { color_choice }
    }

    /// Format the aggregation result for human consumption
    ///
    /// Returns a formatted string suitable for terminal display.
    pub fn format(&self, result: &AggregationResult) -> String {
        let mut output = String::new();

        // Group statuses by rule_id
        let mut current_rule: Option<&str> = None;

        for status in &result.statuses {
            // If this is a new rule, print the rule header
            if current_rule != Some(status.rule_id.as_str()) {
                if current_rule.is_some() {
                    output.push('\n');
                }

                // Count violations for this rule across all regions
                let rule_violations: Vec<&RuleRegionStatus> = result
                    .statuses
                    .iter()
                    .filter(|s| s.rule_id == status.rule_id)
                    .collect();
                let total_violations: u64 = rule_violations.iter().map(|s| s.actual_count).sum();

                // Rule header: no-unwrap (error) [2 violations]
                output.push_str(&format!(
                    "{} [{}]\n\n",
                    status.rule_id.as_str(),
                    if total_violations == 1 {
                        "1 violation".to_string()
                    } else {
                        format!("{} violations", total_violations)
                    }
                ));

                current_rule = Some(status.rule_id.as_str());
            }

            // Print violations for this region (only if there are violations)
            if !status.violations.is_empty() {
                for violation in &status.violations {
                    output.push_str(&format!(
                        "  {}:{}:{}\n",
                        violation.file.display(),
                        violation.line,
                        violation.column
                    ));
                    output.push_str(&format!("      {}\n", violation.snippet.trim()));
                    output.push('\n');
                }
            }
        }

        // Summary section
        if !result.statuses.is_empty() {
            output.push_str("Summary:\n\n");

            for status in &result.statuses {
                let symbol = if status.passed { "✓" } else { "✗" };
                let status_text = if status.passed {
                    format!(
                        "{} violations (budget: {})",
                        status.actual_count, status.budget
                    )
                } else {
                    let exceeded = status.actual_count - status.budget;
                    format!(
                        "{} violations (budget: {}) exceeded by {}",
                        status.actual_count, status.budget, exceeded
                    )
                };

                output.push_str(&format!(
                    "  {} {}: {}\n",
                    symbol,
                    status.rule_id.as_str(),
                    status_text
                ));
            }

            output.push('\n');

            // Final check status
            if result.passed {
                output.push_str("Check PASSED\n");
            } else {
                let rules_exceeded = result.statuses.iter().filter(|s| !s.passed).count();
                output.push_str(&format!(
                    "Check FAILED: {} rule{} exceeded budget\n",
                    rules_exceeded,
                    if rules_exceeded == 1 { "" } else { "s" }
                ));
            }
        } else {
            output.push_str("No violations found\n");
        }

        output
    }

    /// Write the formatted output to stdout with colors
    ///
    /// This method handles colorization and writes directly to stdout.
    pub fn write_to_stdout(&self, result: &AggregationResult) -> io::Result<()> {
        let mut stdout = StandardStream::stdout(self.color_choice);

        // Group statuses by rule_id
        let mut current_rule: Option<&str> = None;

        for status in &result.statuses {
            // If this is a new rule, print the rule header
            if current_rule != Some(status.rule_id.as_str()) {
                if current_rule.is_some() {
                    writeln!(stdout)?;
                }

                // Count violations for this rule across all regions
                let rule_violations: Vec<&RuleRegionStatus> = result
                    .statuses
                    .iter()
                    .filter(|s| s.rule_id == status.rule_id)
                    .collect();
                let total_violations: u64 = rule_violations.iter().map(|s| s.actual_count).sum();

                // Rule header: no-unwrap (error) [2 violations]
                stdout.set_color(ColorSpec::new().set_bold(true))?;
                write!(stdout, "{}", status.rule_id.as_str())?;
                stdout.reset()?;

                write!(stdout, " ")?;

                // Note: Severity information could be displayed here if available
                // Currently, Violation doesn't have a severity field

                stdout.set_color(ColorSpec::new().set_bold(true))?;
                write!(
                    stdout,
                    "[{}]",
                    if total_violations == 1 {
                        "1 violation".to_string()
                    } else {
                        format!("{} violations", total_violations)
                    }
                )?;
                stdout.reset()?;
                writeln!(stdout)?;
                writeln!(stdout)?;

                current_rule = Some(status.rule_id.as_str());
            }

            // Print violations for this region
            if !status.violations.is_empty() {
                for violation in &status.violations {
                    write!(stdout, "  ")?;
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
                    write!(
                        stdout,
                        "{}:{}:{}",
                        violation.file.display(),
                        violation.line,
                        violation.column
                    )?;
                    stdout.reset()?;
                    writeln!(stdout)?;
                    writeln!(stdout, "      {}", violation.snippet.trim())?;
                    writeln!(stdout)?;
                }
            }
        }

        // Summary section
        if !result.statuses.is_empty() {
            stdout.set_color(ColorSpec::new().set_bold(true))?;
            writeln!(stdout, "Summary:")?;
            stdout.reset()?;
            writeln!(stdout)?;

            for status in &result.statuses {
                write!(stdout, "  ")?;

                if status.passed {
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
                    write!(stdout, "✓")?;
                } else {
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
                    write!(stdout, "✗")?;
                }
                stdout.reset()?;

                write!(stdout, " {}: ", status.rule_id.as_str())?;

                if status.passed {
                    stdout.set_color(ColorSpec::new().set_bold(true))?;
                    write!(stdout, "{}", status.actual_count)?;
                    stdout.reset()?;
                    write!(stdout, " violations (budget: ")?;
                    stdout.set_color(ColorSpec::new().set_bold(true))?;
                    write!(stdout, "{}", status.budget)?;
                    stdout.reset()?;
                    writeln!(stdout, ")")?;
                } else {
                    let exceeded = status.actual_count - status.budget;
                    stdout.set_color(ColorSpec::new().set_bold(true))?;
                    write!(stdout, "{}", status.actual_count)?;
                    stdout.reset()?;
                    write!(stdout, " violations (budget: ")?;
                    stdout.set_color(ColorSpec::new().set_bold(true))?;
                    write!(stdout, "{}", status.budget)?;
                    stdout.reset()?;
                    write!(stdout, ") ")?;
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
                    write!(stdout, "exceeded by ")?;
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
                    write!(stdout, "{}", exceeded)?;
                    stdout.reset()?;
                    writeln!(stdout)?;
                }
            }

            writeln!(stdout)?;

            // Final check status
            if result.passed {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
                writeln!(stdout, "Check PASSED")?;
            } else {
                let rules_exceeded = result.statuses.iter().filter(|s| !s.passed).count();
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
                write!(
                    stdout,
                    "Check FAILED: {} rule{} exceeded budget",
                    rules_exceeded,
                    if rules_exceeded == 1 { "" } else { "s" }
                )?;
                stdout.reset()?;
                writeln!(stdout)?;
            }
            stdout.reset()?;
        } else {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            writeln!(stdout, "No violations found")?;
            stdout.reset()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Violation;
    use crate::types::{RegionPath, RuleId};
    use std::path::PathBuf;

    fn create_test_violation(
        rule_id: &str,
        file_path: &str,
        region: &str,
        line: u32,
        snippet: &str,
    ) -> Violation {
        Violation {
            rule_id: RuleId::new(rule_id).unwrap(),
            file: PathBuf::from(file_path),
            line,
            column: 5,
            end_line: line,
            end_column: 15,
            snippet: snippet.to_string(),
            message: "Test violation".to_string(),
            region: RegionPath::new(region),
        }
    }

    fn create_test_status(
        rule_id: &str,
        region: &str,
        actual_count: u64,
        budget: u64,
        violations: Vec<Violation>,
    ) -> RuleRegionStatus {
        RuleRegionStatus {
            rule_id: RuleId::new(rule_id).unwrap(),
            region: RegionPath::new(region),
            actual_count,
            budget,
            passed: actual_count <= budget,
            violations,
        }
    }

    #[test]
    fn test_format_empty_result() {
        let formatter = HumanFormatter::new(ColorChoice::Never);
        let result = AggregationResult {
            statuses: vec![],
            passed: true,
            total_violations: 0,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result);
        assert!(output.contains("No violations found"));
    }

    #[test]
    fn test_format_single_violation_within_budget() {
        let formatter = HumanFormatter::new(ColorChoice::Never);
        let violations = vec![create_test_violation(
            "no-unwrap",
            "src/main.rs",
            "src",
            10,
            ".unwrap()",
        )];
        let status = create_test_status("no-unwrap", "src", 1, 5, violations);
        let result = AggregationResult {
            statuses: vec![status],
            passed: true,
            total_violations: 1,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result);
        assert!(output.contains("no-unwrap"));
        assert!(output.contains("[1 violation]"));
        assert!(output.contains("src/main.rs:10:5"));
        assert!(output.contains(".unwrap()"));
        assert!(output.contains("✓"));
        assert!(output.contains("1 violations (budget: 5)"));
        assert!(output.contains("Check PASSED"));
    }

    #[test]
    fn test_format_multiple_violations_same_rule() {
        let formatter = HumanFormatter::new(ColorChoice::Never);
        let violations = vec![
            create_test_violation("no-unwrap", "src/main.rs", "src", 10, ".unwrap()"),
            create_test_violation("no-unwrap", "src/lib.rs", "src", 20, "result.unwrap()"),
        ];
        let status = create_test_status("no-unwrap", "src", 2, 5, violations);
        let result = AggregationResult {
            statuses: vec![status],
            passed: true,
            total_violations: 2,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result);
        assert!(output.contains("no-unwrap"));
        assert!(output.contains("[2 violations]"));
        assert!(output.contains("src/main.rs:10:5"));
        assert!(output.contains("src/lib.rs:20:5"));
        assert!(output.contains(".unwrap()"));
        assert!(output.contains("result.unwrap()"));
        assert!(output.contains("Check PASSED"));
    }

    #[test]
    fn test_format_violation_over_budget() {
        let formatter = HumanFormatter::new(ColorChoice::Never);
        let violations = vec![
            create_test_violation("no-unwrap", "src/main.rs", "src", 10, ".unwrap()"),
            create_test_violation("no-unwrap", "src/lib.rs", "src", 20, "result.unwrap()"),
        ];
        let status = create_test_status("no-unwrap", "src", 2, 1, violations);
        let result = AggregationResult {
            statuses: vec![status],
            passed: false,
            total_violations: 2,
            violations_over_budget: 1,
        };

        let output = formatter.format(&result);
        assert!(output.contains("no-unwrap"));
        assert!(output.contains("[2 violations]"));
        assert!(output.contains("✗"));
        assert!(output.contains("2 violations (budget: 1) exceeded by 1"));
        assert!(output.contains("Check FAILED: 1 rule exceeded budget"));
    }

    #[test]
    fn test_format_multiple_rules() {
        let formatter = HumanFormatter::new(ColorChoice::Never);
        let violations1 = vec![create_test_violation(
            "no-unwrap",
            "src/main.rs",
            "src",
            10,
            ".unwrap()",
        )];
        let violations2 = vec![create_test_violation(
            "no-todo",
            "src/lib.rs",
            "src",
            20,
            "// TODO: fix",
        )];
        let status1 = create_test_status("no-unwrap", "src", 1, 5, violations1);
        let status2 = create_test_status("no-todo", "src", 1, 3, violations2);
        let result = AggregationResult {
            statuses: vec![status1, status2],
            passed: true,
            total_violations: 2,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result);
        assert!(output.contains("no-unwrap"));
        assert!(output.contains("no-todo"));
        assert!(output.contains("[1 violation]"));
        assert!(output.contains("src/main.rs:10:5"));
        assert!(output.contains("src/lib.rs:20:5"));
        assert!(output.contains("Check PASSED"));
    }

    #[test]
    fn test_format_mixed_pass_fail() {
        let formatter = HumanFormatter::new(ColorChoice::Never);
        let violations1 = vec![create_test_violation(
            "no-unwrap",
            "src/main.rs",
            "src",
            10,
            ".unwrap()",
        )];
        let violations2 = vec![
            create_test_violation("no-todo", "src/lib.rs", "src", 20, "// TODO: fix"),
            create_test_violation("no-todo", "src/util.rs", "src", 30, "// TODO: refactor"),
            create_test_violation("no-todo", "src/test.rs", "src", 40, "// TODO: test"),
            create_test_violation("no-todo", "src/other.rs", "src", 50, "// TODO: cleanup"),
        ];
        let status1 = create_test_status("no-unwrap", "src", 1, 5, violations1);
        let status2 = create_test_status("no-todo", "src", 4, 3, violations2);
        let result = AggregationResult {
            statuses: vec![status1, status2],
            passed: false,
            total_violations: 5,
            violations_over_budget: 1,
        };

        let output = formatter.format(&result);
        assert!(output.contains("no-unwrap"));
        assert!(output.contains("no-todo"));
        assert!(output.contains("[1 violation]"));
        assert!(output.contains("[4 violations]"));
        assert!(output.contains("✓ no-unwrap: 1 violations (budget: 5)"));
        assert!(output.contains("✗ no-todo: 4 violations (budget: 3) exceeded by 1"));
        assert!(output.contains("Check FAILED: 1 rule exceeded budget"));
    }

    #[test]
    fn test_format_multiple_rules_exceeded() {
        let formatter = HumanFormatter::new(ColorChoice::Never);
        let violations1 = vec![
            create_test_violation("no-unwrap", "src/main.rs", "src", 10, ".unwrap()"),
            create_test_violation("no-unwrap", "src/lib.rs", "src", 20, "result.unwrap()"),
        ];
        let violations2 = vec![
            create_test_violation("no-todo", "src/test.rs", "src", 30, "// TODO: fix"),
            create_test_violation("no-todo", "src/util.rs", "src", 40, "// TODO: refactor"),
        ];
        let status1 = create_test_status("no-unwrap", "src", 2, 1, violations1);
        let status2 = create_test_status("no-todo", "src", 2, 0, violations2);
        let result = AggregationResult {
            statuses: vec![status1, status2],
            passed: false,
            total_violations: 4,
            violations_over_budget: 3,
        };

        let output = formatter.format(&result);
        assert!(output.contains("Check FAILED: 2 rules exceeded budget"));
    }

    #[test]
    fn test_write_to_stdout_no_errors() {
        let formatter = HumanFormatter::new(ColorChoice::Never);
        let violations = vec![create_test_violation(
            "no-unwrap",
            "src/main.rs",
            "src",
            10,
            ".unwrap()",
        )];
        let status = create_test_status("no-unwrap", "src", 1, 5, violations);
        let result = AggregationResult {
            statuses: vec![status],
            passed: true,
            total_violations: 1,
            violations_over_budget: 0,
        };

        // This should not panic
        // We can't easily test stdout output in unit tests, but we can verify it doesn't error
        let _ = formatter.write_to_stdout(&result);
    }

    #[test]
    fn test_formatter_with_different_color_choices() {
        // Test that formatters can be created with different color choices
        let _never = HumanFormatter::new(ColorChoice::Never);
        let _always = HumanFormatter::new(ColorChoice::Always);
        let _auto = HumanFormatter::new(ColorChoice::Auto);
    }
}
