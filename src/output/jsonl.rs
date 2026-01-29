#![forbid(unsafe_code)]

//! JSONL output formatter for machine-readable output
//!
//! Outputs one JSON object per line in a deterministic order:
//! 1. All violation records (sorted by rule, file, line)
//! 2. All summary records (sorted by rule, region)
//! 3. One status record

use crate::engine::aggregator::AggregationResult;
use serde::Serialize;
use std::path::PathBuf;

/// JSONL output formatter
///
/// Formats aggregation results as JSON Lines (one JSON object per line).
pub struct JsonlFormatter;

impl JsonlFormatter {
    /// Creates a new JsonlFormatter
    pub fn new() -> Self {
        JsonlFormatter
    }

    /// Format the aggregation result as JSONL
    ///
    /// Returns a string with one JSON object per line:
    /// - First: All violation records (sorted by rule, file, line) - only if verbose is true
    /// - Then: All summary records (sorted by rule, region)
    /// - Finally: One status record
    ///
    /// # Arguments
    ///
    /// * `result` - The aggregation result to format
    /// * `verbose` - If true, output violation records. If false, skip violation records.
    pub fn format(&self, result: &AggregationResult, verbose: bool) -> String {
        let mut output = String::new();

        // Only output violation records if verbose is true
        if verbose {
            // Collect all violations from all statuses
            let mut all_violations: Vec<ViolationRecord> = Vec::new();
            for status in &result.statuses {
                for violation in &status.violations {
                    all_violations.push(ViolationRecord {
                        record_type: "violation".to_string(),
                        rule: status.rule_id.as_str().to_string(),
                        file: violation.file.clone(),
                        line: violation.line,
                        column: violation.column,
                        end_line: violation.end_line,
                        end_column: violation.end_column,
                        snippet: violation.snippet.clone(),
                        message: violation.message.clone(),
                        region: violation.region.as_str().to_string(),
                    });
                }
            }

            // Sort violations by rule, then file, then line
            all_violations.sort_by(|a, b| {
                a.rule
                    .cmp(&b.rule)
                    .then_with(|| a.file.cmp(&b.file))
                    .then_with(|| a.line.cmp(&b.line))
            });

            // Output all violation records
            for violation in all_violations {
                if let Ok(json) = serde_json::to_string(&violation) {
                    output.push_str(&json);
                    output.push('\n');
                }
            }
        }

        // Collect all summary records
        let mut summaries: Vec<SummaryRecord> = Vec::new();
        for status in &result.statuses {
            summaries.push(SummaryRecord {
                record_type: "summary".to_string(),
                rule: status.rule_id.as_str().to_string(),
                region: status.region.as_str().to_string(),
                violations: status.actual_count,
                budget: status.budget,
                status: if status.passed { "pass" } else { "fail" }.to_string(),
            });
        }

        // Sort summaries by rule, then region
        summaries.sort_by(|a, b| a.rule.cmp(&b.rule).then_with(|| a.region.cmp(&b.region)));

        // Output all summary records
        for summary in summaries {
            if let Ok(json) = serde_json::to_string(&summary) {
                output.push_str(&json);
                output.push('\n');
            }
        }

        // Output status record
        let rules_exceeded = result.statuses.iter().filter(|s| !s.passed).count() as u64;
        let status = StatusRecord {
            record_type: "status".to_string(),
            passed: result.passed,
            rules_checked: result.statuses.len() as u64,
            rules_exceeded,
            total_violations: result.total_violations as u64,
        };

        if let Ok(json) = serde_json::to_string(&status) {
            output.push_str(&json);
            output.push('\n');
        }

        output
    }
}

impl Default for JsonlFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Violation record for JSONL output
#[derive(Debug, Serialize)]
struct ViolationRecord {
    #[serde(rename = "type")]
    record_type: String,
    rule: String,
    file: PathBuf,
    line: u32,
    column: u32,
    end_line: u32,
    end_column: u32,
    snippet: String,
    message: String,
    region: String,
}

/// Summary record for JSONL output
#[derive(Debug, Serialize)]
struct SummaryRecord {
    #[serde(rename = "type")]
    record_type: String,
    rule: String,
    region: String,
    violations: u64,
    budget: u64,
    status: String,
}

/// Status record for JSONL output
#[derive(Debug, Serialize)]
struct StatusRecord {
    #[serde(rename = "type")]
    record_type: String,
    passed: bool,
    rules_checked: u64,
    rules_exceeded: u64,
    total_violations: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::aggregator::RuleRegionStatus;
    use crate::rules::Violation;
    use crate::types::{RegionPath, RuleId};
    use std::path::PathBuf;

    fn create_test_violation(
        rule_id: &str,
        file_path: &str,
        region: &str,
        line: u32,
        column: u32,
        snippet: &str,
        message: &str,
    ) -> Violation {
        Violation {
            rule_id: RuleId::new(rule_id).unwrap(),
            file: PathBuf::from(file_path),
            line,
            column,
            end_line: line,
            end_column: column + 10,
            snippet: snippet.to_string(),
            message: message.to_string(),
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
        let formatter = JsonlFormatter::new();
        let result = AggregationResult {
            statuses: vec![],
            passed: true,
            total_violations: 0,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result, true);

        // Should only contain status record
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 1);

        // Parse and verify status record
        let status: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(status["type"], "status");
        assert_eq!(status["passed"], true);
        assert_eq!(status["rules_checked"], 0);
        assert_eq!(status["rules_exceeded"], 0);
        assert_eq!(status["total_violations"], 0);
    }

    #[test]
    fn test_format_single_violation() {
        let formatter = JsonlFormatter::new();
        let violations = vec![create_test_violation(
            "no-unwrap",
            "src/main.rs",
            "src",
            10,
            5,
            ".unwrap()",
            "Disallow .unwrap() calls",
        )];
        let status = create_test_status("no-unwrap", "src", 1, 5, violations);
        let result = AggregationResult {
            statuses: vec![status],
            passed: true,
            total_violations: 1,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result, true);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 3); // 1 violation + 1 summary + 1 status

        // Verify violation record
        let violation: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(violation["type"], "violation");
        assert_eq!(violation["rule"], "no-unwrap");
        assert_eq!(violation["file"], "src/main.rs");
        assert_eq!(violation["line"], 10);
        assert_eq!(violation["column"], 5);
        assert_eq!(violation["end_line"], 10);
        assert_eq!(violation["end_column"], 15);
        assert_eq!(violation["snippet"], ".unwrap()");
        assert_eq!(violation["message"], "Disallow .unwrap() calls");
        assert_eq!(violation["region"], "src");

        // Verify summary record
        let summary: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(summary["type"], "summary");
        assert_eq!(summary["rule"], "no-unwrap");
        assert_eq!(summary["region"], "src");
        assert_eq!(summary["violations"], 1);
        assert_eq!(summary["budget"], 5);
        assert_eq!(summary["status"], "pass");

        // Verify status record
        let status: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
        assert_eq!(status["type"], "status");
        assert_eq!(status["passed"], true);
        assert_eq!(status["rules_checked"], 1);
        assert_eq!(status["rules_exceeded"], 0);
        assert_eq!(status["total_violations"], 1);
    }

    #[test]
    fn test_format_multiple_violations_sorted() {
        let formatter = JsonlFormatter::new();

        // Create violations in unsorted order
        let violations1 = vec![create_test_violation(
            "rule-b", "src/z.rs", "src", 20, 5, "snippet2", "message2",
        )];
        let violations2 = vec![create_test_violation(
            "rule-a", "src/a.rs", "src", 10, 5, "snippet1", "message1",
        )];

        let status1 = create_test_status("rule-b", "src", 1, 5, violations1);
        let status2 = create_test_status("rule-a", "src", 1, 5, violations2);

        let result = AggregationResult {
            statuses: vec![status1, status2],
            passed: true,
            total_violations: 2,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result, true);
        let lines: Vec<&str> = output.lines().collect();

        // Verify violations are sorted by rule, then file, then line
        let v1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        let v2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();

        assert_eq!(v1["rule"], "rule-a");
        assert_eq!(v2["rule"], "rule-b");

        // Verify summaries are sorted by rule, then region
        let s1: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
        let s2: serde_json::Value = serde_json::from_str(lines[3]).unwrap();

        assert_eq!(s1["rule"], "rule-a");
        assert_eq!(s2["rule"], "rule-b");
    }

    #[test]
    fn test_format_violation_over_budget() {
        let formatter = JsonlFormatter::new();
        let violations = vec![
            create_test_violation(
                "no-unwrap",
                "src/main.rs",
                "src",
                10,
                5,
                ".unwrap()",
                "Disallow .unwrap() calls",
            ),
            create_test_violation(
                "no-unwrap",
                "src/lib.rs",
                "src",
                20,
                5,
                "result.unwrap()",
                "Disallow .unwrap() calls",
            ),
        ];
        let status = create_test_status("no-unwrap", "src", 2, 1, violations);
        let result = AggregationResult {
            statuses: vec![status],
            passed: false,
            total_violations: 2,
            violations_over_budget: 1,
        };

        let output = formatter.format(&result, true);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 4); // 2 violations + 1 summary + 1 status

        // Verify summary shows fail status
        let summary: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
        assert_eq!(summary["status"], "fail");
        assert_eq!(summary["violations"], 2);
        assert_eq!(summary["budget"], 1);

        // Verify status record shows failure
        let status: serde_json::Value = serde_json::from_str(lines[3]).unwrap();
        assert_eq!(status["passed"], false);
        assert_eq!(status["rules_exceeded"], 1);
    }

    #[test]
    fn test_format_multiple_rules_and_regions() {
        let formatter = JsonlFormatter::new();

        let violations1 = vec![create_test_violation(
            "no-unwrap",
            "src/main.rs",
            "src",
            10,
            5,
            ".unwrap()",
            "message",
        )];
        let violations2 = vec![create_test_violation(
            "no-unwrap",
            "tests/test.rs",
            "tests",
            20,
            5,
            ".unwrap()",
            "message",
        )];
        let violations3 = vec![create_test_violation(
            "no-todo",
            "src/lib.rs",
            "src",
            30,
            5,
            "// TODO",
            "message",
        )];

        let status1 = create_test_status("no-unwrap", "src", 1, 5, violations1);
        let status2 = create_test_status("no-unwrap", "tests", 1, 10, violations2);
        let status3 = create_test_status("no-todo", "src", 1, 3, violations3);

        let result = AggregationResult {
            statuses: vec![status1, status2, status3],
            passed: true,
            total_violations: 3,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result, true);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 7); // 3 violations + 3 summaries + 1 status

        // Verify violations are properly sorted
        let v1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        let v2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        let v3: serde_json::Value = serde_json::from_str(lines[2]).unwrap();

        assert_eq!(v1["rule"], "no-todo");
        assert_eq!(v2["rule"], "no-unwrap");
        assert_eq!(v3["rule"], "no-unwrap");

        // Verify summaries are properly sorted
        let s1: serde_json::Value = serde_json::from_str(lines[3]).unwrap();
        let s2: serde_json::Value = serde_json::from_str(lines[4]).unwrap();
        let s3: serde_json::Value = serde_json::from_str(lines[5]).unwrap();

        assert_eq!(s1["rule"], "no-todo");
        assert_eq!(s2["rule"], "no-unwrap");
        assert_eq!(s2["region"], "src");
        assert_eq!(s3["rule"], "no-unwrap");
        assert_eq!(s3["region"], "tests");
    }

    #[test]
    fn test_json_validity() {
        let formatter = JsonlFormatter::new();
        let violations = vec![create_test_violation(
            "test-rule",
            "src/test.rs",
            "src",
            1,
            1,
            "test",
            "test message",
        )];
        let status = create_test_status("test-rule", "src", 1, 1, violations);
        let result = AggregationResult {
            statuses: vec![status],
            passed: true,
            total_violations: 1,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result, true);

        // Verify each line is valid JSON
        for line in output.lines() {
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
            assert!(parsed.is_ok(), "Invalid JSON: {}", line);
        }
    }

    #[test]
    fn test_default_implementation() {
        let formatter = JsonlFormatter;
        let result = AggregationResult {
            statuses: vec![],
            passed: true,
            total_violations: 0,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result, true);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_violation_sorting_by_line() {
        let formatter = JsonlFormatter::new();

        // Create violations with same rule and file but different lines
        let violations = vec![
            create_test_violation("rule-a", "src/file.rs", "src", 30, 5, "s3", "m3"),
            create_test_violation("rule-a", "src/file.rs", "src", 10, 5, "s1", "m1"),
            create_test_violation("rule-a", "src/file.rs", "src", 20, 5, "s2", "m2"),
        ];

        let status = create_test_status("rule-a", "src", 3, 5, violations);
        let result = AggregationResult {
            statuses: vec![status],
            passed: true,
            total_violations: 3,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result, true);
        let lines: Vec<&str> = output.lines().collect();

        // Verify violations are sorted by line number
        let v1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        let v2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        let v3: serde_json::Value = serde_json::from_str(lines[2]).unwrap();

        assert_eq!(v1["line"], 10);
        assert_eq!(v2["line"], 20);
        assert_eq!(v3["line"], 30);
    }

    #[test]
    fn test_special_characters_in_paths() {
        let formatter = JsonlFormatter::new();

        // Test with spaces, unicode, and special characters in paths
        let violations = [
            create_test_violation(
                "no-unwrap",
                "src/my file.rs",
                "src",
                10,
                5,
                ".unwrap()",
                "message",
            ),
            create_test_violation(
                "no-todo",
                "src/日本語.rs",
                "src",
                20,
                5,
                "// TODO",
                "message",
            ),
            create_test_violation(
                "no-panic",
                "src/file's.rs",
                "src",
                30,
                5,
                "panic!",
                "message",
            ),
        ];

        let status1 = create_test_status("no-unwrap", "src", 1, 5, vec![violations[0].clone()]);
        let status2 = create_test_status("no-todo", "src", 1, 5, vec![violations[1].clone()]);
        let status3 = create_test_status("no-panic", "src", 1, 5, vec![violations[2].clone()]);

        let result = AggregationResult {
            statuses: vec![status1, status2, status3],
            passed: true,
            total_violations: 3,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result, true);

        // Verify all lines are valid JSON
        for line in output.lines() {
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
            assert!(parsed.is_ok(), "Invalid JSON: {}", line);
        }

        // Verify special characters in paths are properly JSON-encoded
        // Violations are sorted by rule, then file, then line
        let lines: Vec<&str> = output.lines().collect();
        let v1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        let v2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        let v3: serde_json::Value = serde_json::from_str(lines[2]).unwrap();

        // Sorted order: no-panic, no-todo, no-unwrap
        assert_eq!(v1["rule"], "no-panic");
        assert_eq!(v1["file"], "src/file's.rs");
        assert_eq!(v2["rule"], "no-todo");
        assert_eq!(v2["file"], "src/日本語.rs");
        assert_eq!(v3["rule"], "no-unwrap");
        assert_eq!(v3["file"], "src/my file.rs");
    }

    #[test]
    fn test_special_characters_in_snippets() {
        let formatter = JsonlFormatter::new();

        // Test with various special characters in snippets
        let violations = vec![
            create_test_violation(
                "test",
                "src/test.rs",
                "src",
                1,
                1,
                "\"hello\\nworld\"",
                "newline in snippet",
            ),
            create_test_violation(
                "test",
                "src/test.rs",
                "src",
                2,
                1,
                "emoji: 🦀",
                "emoji in snippet",
            ),
            create_test_violation(
                "test",
                "src/test.rs",
                "src",
                3,
                1,
                "{\"key\": \"value\"}",
                "json in snippet",
            ),
            create_test_violation(
                "test",
                "src/test.rs",
                "src",
                4,
                1,
                "tab:\there",
                "tab character",
            ),
        ];

        let status = create_test_status("test", "src", 4, 10, violations);
        let result = AggregationResult {
            statuses: vec![status],
            passed: true,
            total_violations: 4,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result, true);

        // Verify all lines are valid JSON
        for line in output.lines() {
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
            assert!(parsed.is_ok(), "Invalid JSON: {}", line);
        }

        // Verify special characters are properly escaped
        let lines: Vec<&str> = output.lines().collect();
        let v1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        let v2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        let v3: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
        let v4: serde_json::Value = serde_json::from_str(lines[3]).unwrap();

        assert_eq!(v1["snippet"], "\"hello\\nworld\"");
        assert_eq!(v2["snippet"], "emoji: 🦀");
        assert_eq!(v3["snippet"], "{\"key\": \"value\"}");
        assert_eq!(v4["snippet"], "tab:\there");
    }

    #[test]
    fn test_deterministic_output() {
        let formatter = JsonlFormatter::new();

        // Create a complex result with multiple violations
        let violations1 = [
            create_test_violation("rule-b", "src/z.rs", "src", 20, 5, "snippet2", "message2"),
            create_test_violation("rule-a", "src/a.rs", "src", 10, 5, "snippet1", "message1"),
        ];
        let violations2 = vec![create_test_violation(
            "rule-c",
            "tests/test.rs",
            "tests",
            30,
            5,
            "snippet3",
            "message3",
        )];

        let status1 = create_test_status("rule-b", "src", 1, 5, vec![violations1[0].clone()]);
        let status2 = create_test_status("rule-a", "src", 1, 5, vec![violations1[1].clone()]);
        let status3 = create_test_status("rule-c", "tests", 1, 5, violations2);

        let result = AggregationResult {
            statuses: vec![status1, status2, status3],
            passed: true,
            total_violations: 3,
            violations_over_budget: 0,
        };

        // Format the same result multiple times
        let output1 = formatter.format(&result, true);
        let output2 = formatter.format(&result, true);
        let output3 = formatter.format(&result, true);

        // All outputs should be byte-for-byte identical
        assert_eq!(output1, output2);
        assert_eq!(output2, output3);

        // Verify the output is sorted correctly
        let lines: Vec<&str> = output1.lines().collect();

        // First 3 lines should be violations sorted by rule, file, line
        let v1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        let v2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        let v3: serde_json::Value = serde_json::from_str(lines[2]).unwrap();

        assert_eq!(v1["rule"], "rule-a");
        assert_eq!(v2["rule"], "rule-b");
        assert_eq!(v3["rule"], "rule-c");

        // Next 3 lines should be summaries sorted by rule, region
        let s1: serde_json::Value = serde_json::from_str(lines[3]).unwrap();
        let s2: serde_json::Value = serde_json::from_str(lines[4]).unwrap();
        let s3: serde_json::Value = serde_json::from_str(lines[5]).unwrap();

        assert_eq!(s1["type"], "summary");
        assert_eq!(s1["rule"], "rule-a");
        assert_eq!(s2["type"], "summary");
        assert_eq!(s2["rule"], "rule-b");
        assert_eq!(s3["type"], "summary");
        assert_eq!(s3["rule"], "rule-c");

        // Last line should be status
        let status: serde_json::Value = serde_json::from_str(lines[6]).unwrap();
        assert_eq!(status["type"], "status");
    }

    #[test]
    fn test_empty_and_whitespace_snippets() {
        let formatter = JsonlFormatter::new();

        let violations = vec![
            create_test_violation("test", "src/test.rs", "src", 1, 1, "", "empty"),
            create_test_violation("test", "src/test.rs", "src", 2, 1, "   ", "whitespace only"),
            create_test_violation(
                "test",
                "src/test.rs",
                "src",
                3,
                1,
                "\n\n\n",
                "newlines only",
            ),
        ];

        let status = create_test_status("test", "src", 3, 10, violations);
        let result = AggregationResult {
            statuses: vec![status],
            passed: true,
            total_violations: 3,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result, true);

        // Verify all lines are valid JSON
        for line in output.lines() {
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
            assert!(parsed.is_ok(), "Invalid JSON: {}", line);
        }

        // Verify snippets are preserved as-is
        let lines: Vec<&str> = output.lines().collect();
        let v1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        let v2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        let v3: serde_json::Value = serde_json::from_str(lines[2]).unwrap();

        assert_eq!(v1["snippet"], "");
        assert_eq!(v2["snippet"], "   ");
        assert_eq!(v3["snippet"], "\n\n\n");
    }

    #[test]
    fn test_long_snippet_json_encoding() {
        let formatter = JsonlFormatter::new();

        // Create a very long snippet
        let long_snippet = "a".repeat(10000);
        let violations = vec![create_test_violation(
            "test-rule",
            "src/test.rs",
            "src",
            1,
            1,
            &long_snippet,
            "long snippet",
        )];

        let status = create_test_status("test-rule", "src", 1, 5, violations);
        let result = AggregationResult {
            statuses: vec![status],
            passed: true,
            total_violations: 1,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result, true);

        // Verify the line is valid JSON
        let lines: Vec<&str> = output.lines().collect();
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(lines[0]);
        assert!(parsed.is_ok());

        let violation: serde_json::Value = parsed.unwrap();
        assert_eq!(violation["snippet"], long_snippet);
    }

    #[test]
    fn test_special_characters_in_messages() {
        let formatter = JsonlFormatter::new();

        let violations = vec![
            create_test_violation(
                "test",
                "src/test.rs",
                "src",
                1,
                1,
                "snippet",
                "message with \"quotes\"",
            ),
            create_test_violation(
                "test",
                "src/test.rs",
                "src",
                2,
                1,
                "snippet",
                "message with 'apostrophe's",
            ),
            create_test_violation(
                "test",
                "src/test.rs",
                "src",
                3,
                1,
                "snippet",
                "message\nwith\nnewlines",
            ),
        ];

        let status = create_test_status("test", "src", 3, 10, violations);
        let result = AggregationResult {
            statuses: vec![status],
            passed: true,
            total_violations: 3,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result, true);

        // Verify all lines are valid JSON
        for line in output.lines() {
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
            assert!(parsed.is_ok(), "Invalid JSON: {}", line);
        }

        // Verify messages are properly escaped
        let lines: Vec<&str> = output.lines().collect();
        let v1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        let v2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        let v3: serde_json::Value = serde_json::from_str(lines[2]).unwrap();

        assert_eq!(v1["message"], "message with \"quotes\"");
        assert_eq!(v2["message"], "message with 'apostrophe's");
        assert_eq!(v3["message"], "message\nwith\nnewlines");
    }

    #[test]
    fn test_region_status_with_no_violations() {
        let formatter = JsonlFormatter::new();

        // Create a status with no violations but positive budget
        let status = create_test_status("no-unwrap", "src", 0, 5, vec![]);
        let result = AggregationResult {
            statuses: vec![status],
            passed: true,
            total_violations: 0,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result, true);
        let lines: Vec<&str> = output.lines().collect();

        // Should have 2 lines: 1 summary + 1 status (no violation records)
        assert_eq!(lines.len(), 2);

        // Verify summary record
        let summary: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(summary["type"], "summary");
        assert_eq!(summary["rule"], "no-unwrap");
        assert_eq!(summary["violations"], 0);
        assert_eq!(summary["budget"], 5);
        assert_eq!(summary["status"], "pass");

        // Verify status record
        let status: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(status["type"], "status");
        assert_eq!(status["passed"], true);
    }

    #[test]
    fn test_violation_sorting_by_file() {
        let formatter = JsonlFormatter::new();

        // Create violations with same rule but different files
        let violations = vec![
            create_test_violation("rule-a", "src/z.rs", "src", 10, 5, "s1", "m1"),
            create_test_violation("rule-a", "src/a.rs", "src", 10, 5, "s2", "m2"),
            create_test_violation("rule-a", "src/m.rs", "src", 10, 5, "s3", "m3"),
        ];

        let status = create_test_status("rule-a", "src", 3, 5, violations);
        let result = AggregationResult {
            statuses: vec![status],
            passed: true,
            total_violations: 3,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result, true);
        let lines: Vec<&str> = output.lines().collect();

        // Verify violations are sorted by file path
        let v1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        let v2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        let v3: serde_json::Value = serde_json::from_str(lines[2]).unwrap();

        assert_eq!(v1["file"], "src/a.rs");
        assert_eq!(v2["file"], "src/m.rs");
        assert_eq!(v3["file"], "src/z.rs");
    }

    #[test]
    fn test_format_non_verbose_hides_violation_records() {
        // Test that when verbose=false, "type":"violation" records are not output
        let formatter = JsonlFormatter::new();
        let violations = vec![
            create_test_violation(
                "no-unwrap",
                "src/main.rs",
                "src",
                10,
                5,
                ".unwrap()",
                "Disallow .unwrap() calls",
            ),
            create_test_violation(
                "no-unwrap",
                "src/lib.rs",
                "src",
                20,
                5,
                "result.unwrap()",
                "Disallow .unwrap() calls",
            ),
        ];
        let status = create_test_status("no-unwrap", "src", 2, 5, violations);
        let result = AggregationResult {
            statuses: vec![status],
            passed: true,
            total_violations: 2,
            violations_over_budget: 0,
        };

        let output = formatter.format(&result, false);

        // Parse each line as JSON
        let lines: Vec<&str> = output.lines().collect();

        // Assert no lines have "type":"violation"
        for line in &lines {
            let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
            assert_ne!(parsed["type"], "violation");
        }

        // Assert there ARE lines with "type":"summary"
        let has_summary = lines.iter().any(|line| {
            let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
            parsed["type"] == "summary"
        });
        assert!(has_summary);

        // Assert there IS a line with "type":"status"
        let has_status = lines.iter().any(|line| {
            let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
            parsed["type"] == "status"
        });
        assert!(has_status);
    }

    #[test]
    fn test_format_non_verbose_preserves_summary_records() {
        // Test that summary and status records are still output when verbose=false
        let formatter = JsonlFormatter::new();

        // Create multiple rules with violations
        let violations1 = vec![create_test_violation(
            "no-unwrap",
            "src/main.rs",
            "src",
            10,
            5,
            ".unwrap()",
            "message",
        )];
        let violations2 = vec![
            create_test_violation("no-todo", "src/lib.rs", "src", 20, 5, "// TODO", "message"),
            create_test_violation("no-todo", "src/util.rs", "src", 30, 5, "// TODO", "message"),
        ];

        let status1 = create_test_status("no-unwrap", "src", 1, 5, violations1);
        let status2 = create_test_status("no-todo", "src", 2, 1, violations2);

        let result = AggregationResult {
            statuses: vec![status1, status2],
            passed: false,
            total_violations: 3,
            violations_over_budget: 1,
        };

        let output = formatter.format(&result, false);
        let lines: Vec<&str> = output.lines().collect();

        // Should have 3 lines: 2 summaries + 1 status (no violation records)
        assert_eq!(lines.len(), 3);

        // Verify first two lines are summaries
        let summary1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        let summary2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(summary1["type"], "summary");
        assert_eq!(summary2["type"], "summary");

        // Verify last line is status
        let status: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
        assert_eq!(status["type"], "status");
        assert_eq!(status["passed"], false);
        assert_eq!(status["rules_checked"], 2);
        assert_eq!(status["rules_exceeded"], 1);
        assert_eq!(status["total_violations"], 3);
    }
}
