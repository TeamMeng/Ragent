//! Tool definitions for the agent.

pub mod calculator;
pub mod code_sandbox;
pub mod web_search;

pub use calculator::{CalculatorTool, CalculatorInput, CalculatorOutput};
pub use web_search::{WebSearchTool, WebSearchInput, WebSearchOutput};
pub use code_sandbox::{CodeSandboxTool, CodeSandboxInput, CodeSandboxOutput};

use serde::{Deserialize, Serialize};

/// Unified tool call input.
#[derive(Debug, Deserialize)]
#[serde(tag = "tool")]
pub enum ToolCall {
    #[serde(rename = "calculator")]
    Calculator { expression: String },
    #[serde(rename = "web_search")]
    WebSearch { query: String },
    #[serde(rename = "code_sandbox")]
    CodeSandbox { language: String, code: String },
}

/// Unified tool result.
#[derive(Debug, Serialize)]
#[serde(tag = "tool")]
pub enum ToolResult {
    Calculator { result: f64, expression: String },
    WebSearch { results: Vec<serde_json::Value>, query: String },
    CodeSandbox { stdout: String, stderr: String, exit_code: i32, timed_out: bool },
}

impl ToolResult {
    pub fn from_calculator(out: CalculatorOutput) -> Self {
        Self::Calculator { result: out.result, expression: out.expression }
    }
    pub fn from_search(out: WebSearchOutput) -> Self {
        Self::WebSearch { results: vec![], query: out.query }
    }
    pub fn from_sandbox(out: CodeSandboxOutput) -> Self {
        Self::CodeSandbox { stdout: out.stdout, stderr: out.stderr, exit_code: out.exit_code, timed_out: out.timed_out }
    }
}
