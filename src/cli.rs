use clap::{Args, Parser, command};
use serde::Serialize;

#[derive(Debug, Parser, Serialize, Clone)]
#[command(version, about = "Run jest-lua tests from the command line")]
#[serde(rename_all = "camelCase")]
pub struct Cli {
    /// A list of Roblox paths for Jest Lua to discover.
    #[arg(short, long, required = true, value_delimiter = ',')]
    projects: Vec<String>,

    /// Timeout for the server to receive results in seconds.
    #[arg(short, long, default_value_t = 30)]
    pub server_timeout: u64,

    #[command(flatten, next_help_heading = "runCLI options")]
    pub options: JestOptions,
}

#[derive(Debug, Args, Serialize, Clone)]
#[command(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct JestOptions {
    /// Automatically clear mock calls, instances, contexts and results before every test.
    /// Equivalent to calling jest.clearAllMocks() before each test. This does not remove any mock implementation that may have been provided.
    #[arg(long, verbatim_doc_comment)]
    clear_mocks: Option<bool>,

    /// Use this flag to show full diffs and errors instead of a patch.
    #[arg(long)]
    expand: Option<bool>,

    /// Disables stack trace in test results output.
    #[arg(long)]
    no_stack_trace: Option<bool>,

    /// Changes how jest.spyOn() overwrites methods in the spied object, making it behave like older versions of Jest.
    /// When oldFunctionSpying = true, it will overwrite the spied method with a mock object. (old behaviour)
    /// When oldFunctionSpying = false, it will overwrite the spied method with a regular Lua function. (new behaviour)
    #[arg(long, verbatim_doc_comment)]
    old_function_spying: Option<bool>,

    /// Allows the test suite to pass when no files are found.
    #[arg(long)]
    pass_with_no_tests: Option<bool>,

    /// Automatically reset mock state before every test.
    /// Equivalent to calling jest.resetAllMocks() before each test. This will lead to any mocks having their fake implementations removed but does not restore their initial implementation.
    #[arg(long, verbatim_doc_comment)]
    reset_mocks: Option<bool>,

    /// The glob patterns Jest uses to detect test files.
    #[arg(long, value_delimiter = ',')]
    test_match: Option<Vec<String>>,

    /// Run only tests with a name that matches the regex.
    /// For example, suppose you want to run only tests related to authorization which will have names like "GET /api/posts with auth", then you can use testNamePattern = "auth".
    /// The regex is matched against the full name, which is a combination of the test name and all its surrounding describe blocks.
    #[arg(long, verbatim_doc_comment)]
    test_name_pattern: Option<String>,

    /// An array of regexp pattern strings that are tested against all tests paths before executing the test.
    /// Contrary to testPathPattern, it will only run those tests with a path that does not match with the provided regexp expressions.
    #[arg(long, verbatim_doc_comment)]
    test_path_ignore_patterns: Option<Vec<String>>,

    /// A regexp pattern string that is matched against all tests paths before executing the test.
    #[arg(long)]
    test_path_pattern: Option<Option<String>>,

    /// Default timeout of a test in milliseconds.
    #[arg(long)]
    test_timeout: Option<u32>,

    /// Display individual test results with the test suite hierarchy.
    #[arg(long)]
    pub verbose: Option<bool>,
}
