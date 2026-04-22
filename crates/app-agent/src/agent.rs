//! AI agent module — calls Ollama REST API directly.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

use crate::tools::{CalculatorTool, WebSearchTool, CodeSandboxTool, ToolCall, ToolResult};

/// Configuration for the agent.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub ollama_base_url: String,
    pub model: String,
    pub system_prompt: String,
    pub enable_tools: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            ollama_base_url: "http://localhost:11434".into(),
            model: "llama3".into(),
            system_prompt: "You are a helpful AI assistant. Use tools when needed.".into(),
            enable_tools: true,
        }
    }
}

/// Ollama chat message.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

/// Ollama API request.
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    tools: Option<Vec<serde_json::Value>>,
}

/// Ollama API response.
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: Option<OllamaMessage>,
    response: Option<String>,
    done: bool,
}

/// A chat agent that talks to Ollama.
pub struct ChatAgent {
    client: reqwest::Client,
    config: AgentConfig,
    calculator: CalculatorTool,
    web_search: WebSearchTool,
    code_sandbox: CodeSandboxTool,
}

impl ChatAgent {
    pub fn new(config: AgentConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config,
            calculator: CalculatorTool::new(),
            web_search: WebSearchTool::new(),
            code_sandbox: CodeSandboxTool::new(),
        }
    }

    /// Send a chat message and get a response.
    pub async fn chat(&self, history: Vec<OllamaMessage>, user_message: &str) -> Result<String> {
        info!(model = %self.config.model, "Sending chat request to Ollama");

        let mut messages = vec![OllamaMessage {
            role: "system".into(),
            content: self.config.system_prompt.clone(),
        }];
        messages.extend(history);
        messages.push(OllamaMessage {
            role: "user".into(),
            content: user_message.into(),
        });

        let body = OllamaRequest {
            model: self.config.model.clone(),
            messages,
            stream: false,
            tools: None,
        };

        let url = format!("{}/api/chat", self.config.ollama_base_url);
        let resp = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Failed to connect to Ollama")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Ollama API error {}: {}", status, text);
        }

        let data: OllamaResponse = resp.json().await.context("Failed to parse Ollama response")?;
        let text = data.message
            .map(|m| m.content)
            .or(data.response)
            .unwrap_or_default();

        info!(response_len = text.len(), "Received response from Ollama");
        Ok(text)
    }

    /// Execute a tool call by name.
    pub async fn call_tool(&self, name: &str, input: serde_json::Value) -> Result<ToolResult> {
        match name {
            "calculator" => {
                let args: ToolCall = serde_json::from_value(input)?;
                let out = self.calculator.call(args).await?;
                Ok(ToolResult::from_calculator(out))
            }
            "web_search" => {
                let args = serde_json::from_value(input)
                    .unwrap_or(serde_json::json!({"query": ""}));
                let out = self.web_search.call(args).await?;
                Ok(ToolResult::from_search(out))
            }
            "code_sandbox" => {
                let args = serde_json::from_value(input)?;
                let out = self.code_sandbox.call(args).await?;
                Ok(ToolResult::from_sandbox(out))
            }
            _ => anyhow::bail!("Unknown tool: {}", name),
        }
    }
}
