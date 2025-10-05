use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub results: Results,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Results {
    pub success: bool,
    pub num_failed_test_suites: u32,
    pub num_failed_tests: u32,
    pub num_passed_test_suites: u32,
    pub num_passed_tests: u32,
    pub num_pending_tests: u32,
    pub num_todo_tests: u32,
    pub num_pending_test_suites: u32,
    pub num_runtime_error_test_suites: u32,
    pub num_total_test_suites: u32,
    pub num_total_tests: u32,
    pub test_results: Vec<TestResult>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    pub test_file_path: String,
    pub num_failing_tests: u32,
    pub test_results: Vec<AssertionResult>,
    pub perf_stats: PerfStats,
    pub failure_message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssertionResult {
    pub title: String,
    pub status: Status,
    pub failure_messages: Vec<String>,
    pub duration: Option<u64>,
    pub ancestor_titles: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Passed,
    Failed,
    Skipped,
    Pending,
    Todo,
    Disabled,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerfStats {
    pub runtime: u64,
}
