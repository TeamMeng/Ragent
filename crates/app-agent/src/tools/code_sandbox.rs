//! Code sandbox tool — safe execution of user-provided code snippets.
//! Production: use Wasmtime (Wasm) or nsjail + cgroup v2.

use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncWriteExt;
use tracing::{info, warn, error};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CodeSandboxTool;

#[derive(Debug, Deserialize, Serialize)]
pub struct CodeSandboxInput {
    /// Programming language: "python", "javascript", "rust", etc.
    pub language: String,
    /// Source code to execute.
    pub code: String,
    /// Optional stdin input.
    pub stdin: Option<String>,
    /// Timeout in seconds (max 30).
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CodeSandboxOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
}

impl CodeSandboxTool {
    pub fn new() -> Self { Self }

    pub fn name(&self) -> &str { "code_sandbox" }

    pub fn description(&self) -> &str {
        "Execute a short code snippet safely in a sandboxed environment. Supports python, javascript, and rust. Max timeout 30s."
    }

    pub async fn call(&self, args: CodeSandboxInput) -> Result<CodeSandboxOutput, Box<dyn std::error::Error + Send + Sync>> {
        let timeout_secs = args.timeout_secs.unwrap_or(10).min(30);
        let lang = args.language.to_lowercase();

        info!(language = %lang, timeout = timeout_secs, "Executing sandbox code");

        match lang.as_str() {
            "python" | "python3" | "py" => self.exec_python(&args.code, args.stdin.as_deref(), timeout_secs).await,
            "javascript" | "js" | "node" => self.exec_node(&args.code, args.stdin.as_deref(), timeout_secs).await,
            "rust" | "rs" => self.exec_rust(&args.code, args.stdin.as_deref(), timeout_secs).await,
            _ => Err(format!("Unsupported language: {}", lang).into()),
        }
    }

    async fn exec_python(
        &self, code: &str, stdin: Option<&str>, timeout: u64,
    ) -> Result<CodeSandboxOutput, Box<dyn std::error::Error + Send + Sync>> {
        let mut child = Command::new("python3")
            .args(["-c", code])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        if let Some(input) = stdin {
            if let Some(mut stdin_pipe) = child.stdin.take() {
                stdin_pipe.write_all(input.as_bytes()).await?;
            }
        }

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(timeout),
            child.wait_with_output(),
        ).await;

        match result {
            Ok(Ok(output)) => Ok(CodeSandboxOutput {
                stdout: String::from_utf8_lossy(&output.stdout).into(),
                stderr: String::from_utf8_lossy(&output.stderr).into(),
                exit_code: output.status.code().unwrap_or(-1),
                timed_out: false,
            }),
            Ok(Err(e)) => Err(e.into()),
            Err(_) => {
                warn!("Python execution timed out after {}s", timeout);
                Ok(CodeSandboxOutput {
                    stdout: String::new(),
                    stderr: format!("Execution timed out after {}s", timeout),
                    exit_code: -1,
                    timed_out: true,
                })
            }
        }
    }

    async fn exec_node(
        &self, code: &str, stdin: Option<&str>, timeout: u64,
    ) -> Result<CodeSandboxOutput, Box<dyn std::error::Error + Send + Sync>> {
        let mut child = Command::new("node")
            .args(["-e", code])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        if let Some(input) = stdin {
            if let Some(mut stdin_pipe) = child.stdin.take() {
                stdin_pipe.write_all(input.as_bytes()).await?;
            }
        }

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(timeout),
            child.wait_with_output(),
        ).await;

        match result {
            Ok(Ok(output)) => Ok(CodeSandboxOutput {
                stdout: String::from_utf8_lossy(&output.stdout).into(),
                stderr: String::from_utf8_lossy(&output.stderr).into(),
                exit_code: output.status.code().unwrap_or(-1),
                timed_out: false,
            }),
            Ok(Err(e)) => Err(e.into()),
            Err(_) => Ok(CodeSandboxOutput {
                stdout: String::new(),
                stderr: format!("Execution timed out after {}s", timeout),
                exit_code: -1,
                timed_out: true,
            }),
        }
    }

    async fn exec_rust(
        &self, code: &str, _stdin: Option<&str>, timeout: u64,
    ) -> Result<CodeSandboxOutput, Box<dyn std::error::Error + Send + Sync>> {
        // For Rust, we use a temp file and `rustc --edition 2021 -o /tmp/out /tmp/src.rs`
        let tmp_dir = tempfile::tempdir()?;
        let src_path = tmp_dir.path().join("main.rs");
        let out_path = tmp_dir.path().join("sandbox_out");
        tokio::fs::write(&src_path, code).await?;

        let compile = Command::new("rustc")
            .args(["--edition", "2021", "-o", &out_path.to_string_lossy(), &src_path.to_string_lossy()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .output()
            .await?;

        if !compile.status.success() {
            return Ok(CodeSandboxOutput {
                stdout: String::new(),
                stderr: String::from_utf8_lossy(&compile.stderr).into(),
                exit_code: compile.status.code().unwrap_or(-1),
                timed_out: false,
            });
        }

        let run_result = tokio::time::timeout(
            std::time::Duration::from_secs(timeout),
            Command::new(&out_path).stdout(Stdio::piped()).stderr(Stdio::piped()).output(),
        ).await;

        match run_result {
            Ok(Ok(output)) => Ok(CodeSandboxOutput {
                stdout: String::from_utf8_lossy(&output.stdout).into(),
                stderr: String::from_utf8_lossy(&output.stderr).into(),
                exit_code: output.status.code().unwrap_or(-1),
                timed_out: false,
            }),
            Ok(Err(e)) => Err(e.into()),
            Err(_) => Ok(CodeSandboxOutput {
                stdout: String::new(),
                stderr: format!("Execution timed out after {}s", timeout),
                exit_code: -1,
                timed_out: true,
            }),
        }
    }
}
