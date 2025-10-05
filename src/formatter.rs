use crate::output::{Output, Results, Status, TestResult};
use colored::*;
use std::fmt::{self, Write as _};

pub struct JestFormatter {
    verbose: bool,
}

impl JestFormatter {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn format_output(&self, output: &Output) -> String {
        let mut buf = String::new();

        self.write_test_results(&mut buf, &output.results.test_results)
            .unwrap();
        writeln!(buf).ok();

        self.write_summary(&mut buf, &output.results).unwrap();

        buf
    }

    fn write_summary(&self, buf: &mut String, results: &Results) -> fmt::Result {
        // Suites
        writeln!(buf, "{}", "Test Suites:".bold())?;
        write!(buf, "  ")?;

        let total_failed_suites =
            results.num_failed_test_suites + results.num_runtime_error_test_suites;

        let mut suite_parts = Vec::new();

        if total_failed_suites > 0 {
            suite_parts.push(format!(
                "{} failed",
                total_failed_suites.to_string().red().bold()
            ));
        }
        if results.num_pending_test_suites > 0 {
            suite_parts.push(format!(
                "{} pending",
                results.num_pending_test_suites.to_string().yellow()
            ));
        }
        if results.num_passed_test_suites > 0 {
            suite_parts.push(format!(
                "{} passed",
                results.num_passed_test_suites.to_string().green()
            ));
        }

        writeln!(
            buf,
            "{}, {} total",
            suite_parts.join(", "),
            results.num_total_test_suites
        )?;

        // Tests
        writeln!(buf, "{}", "Tests:".bold())?;
        write!(buf, "  ")?;

        let mut test_parts = Vec::new();

        if results.num_failed_tests > 0 {
            test_parts.push(format!(
                "{} failed",
                results.num_failed_tests.to_string().red().bold()
            ));
        }
        if results.num_pending_tests > 0 {
            test_parts.push(format!(
                "{} pending",
                results.num_pending_tests.to_string().yellow()
            ));
        }
        if results.num_todo_tests > 0 {
            test_parts.push(format!(
                "{} todo",
                results.num_todo_tests.to_string().blue()
            ));
        }
        if results.num_passed_tests > 0 {
            test_parts.push(format!(
                "{} passed",
                results.num_passed_tests.to_string().green()
            ));
        }

        writeln!(
            buf,
            "{}, {} total",
            test_parts.join(", "),
            results.num_total_tests
        )?;

        // Duration
        let total_duration = self.total_duration(&results.test_results);
        writeln!(buf, "{} {}ms", "Time:".bold(), total_duration)?;

        Ok(())
    }

    fn write_test_results(&self, buf: &mut String, test_results: &[TestResult]) -> fmt::Result {
        for test in test_results {
            self.write_test_file(buf, test)?;
        }
        Ok(())
    }

    fn write_test_file(&self, buf: &mut String, test_file: &TestResult) -> fmt::Result {
        let failed = test_file.num_failing_tests > 0;
        let (icon, color) = if failed {
            ("×", "red")
        } else {
            ("✓", "green")
        };

        writeln!(
            buf,
            "{} {} {}",
            icon.color(color).bold(),
            test_file.test_file_path.bold(),
            format!("({}ms)", test_file.perf_stats.runtime).dimmed()
        )?;

        if let Some(msg) = &test_file.failure_message
            && test_file.test_results.is_empty()
        {
            writeln!(buf, "  {} Test suite failed to run", "●".red().bold())?;
            writeln!(buf)?;
            self.write_indented(buf, msg, 2, Some(Color::BrightRed))?;
            return Ok(());
        }

        let mut current_ancestors: Vec<String> = Vec::new();

        for case in &test_file.test_results {
            let show_test = matches!(case.status, Status::Failed) || self.verbose;
            if !show_test {
                continue;
            }

            if case.ancestor_titles != current_ancestors {
                let mut common_len = 0;
                for (i, (old, new)) in current_ancestors
                    .iter()
                    .zip(case.ancestor_titles.iter())
                    .enumerate()
                {
                    if old == new {
                        common_len = i + 1;
                    } else {
                        break;
                    }
                }

                for (i, ancestor) in case.ancestor_titles.iter().enumerate().skip(common_len) {
                    let indent = "    ".repeat(i + 1);
                    writeln!(buf, "{}{}", indent, ancestor)?;
                }

                current_ancestors = case.ancestor_titles.clone();
            }

            let test_indent = "    ".repeat(case.ancestor_titles.len() + 1);

            match case.status {
                Status::Failed => {
                    let duration = case
                        .duration
                        .map(|d| format!(" ({}ms)", d))
                        .unwrap_or_default();

                    writeln!(
                        buf,
                        "{}{}{}{}",
                        test_indent,
                        "×".red().bold(),
                        format!(" {}", case.title).red(),
                        duration.dimmed()
                    )?;

                    for msg in &case.failure_messages {
                        let error_indent = "    ".repeat(case.ancestor_titles.len() + 2);
                        self.write_indented(buf, msg, error_indent.len(), Some(Color::BrightRed))?;
                    }
                }
                Status::Passed => {
                    let duration = case
                        .duration
                        .map(|d| format!(" ({}ms)", d))
                        .unwrap_or_default();

                    writeln!(
                        buf,
                        "{}{} {}{}",
                        test_indent,
                        "✓".green(),
                        case.title,
                        duration.dimmed()
                    )?;
                }
                Status::Pending => {
                    writeln!(
                        buf,
                        "{}{} {}",
                        test_indent,
                        "○".yellow(),
                        case.title.yellow()
                    )?;
                }
                Status::Todo => {
                    writeln!(buf, "{}{} {}", test_indent, "✎".blue(), case.title.blue())?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn write_indented(
        &self,
        buf: &mut String,
        text: &str,
        indent: usize,
        color: Option<Color>,
    ) -> fmt::Result {
        let prefix = " ".repeat(indent);

        for line in text.lines() {
            let line = format!("{prefix}{line}");

            if let Some(c) = color {
                writeln!(buf, "{}", line.color(c))?;
            } else {
                writeln!(buf, "{line}")?;
            }
        }

        Ok(())
    }

    fn total_duration(&self, results: &[TestResult]) -> u64 {
        results.iter().map(|r| r.perf_stats.runtime).sum()
    }
}
