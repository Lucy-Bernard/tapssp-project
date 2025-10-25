/*!
 * SANDBOX EXECUTOR
 *
 * In the Python version, this uses RestrictedPython to execute AI-generated Python code.
 * In Rust, for security reasons, we don't execute arbitrary code. Instead, we use
 * a structured approach where the AI returns JSON that we parse and validate.
 *
 * This module provides validation and execution of AI diagnosis actions.
 */

use anyhow::{Context, Result};
use serde_json::Value as JsonValue;

use crate::domain::enums::DiagnosisAction;

pub struct SandboxExecutor;

#[derive(Debug)]
pub struct ExecutionResult {
    pub action: DiagnosisAction,
    pub payload: JsonValue,
}

impl SandboxExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Validate and parse AI-generated response into an execution result
    ///
    /// In Python version: Executes AI-generated Python code in RestrictedPython sandbox
    /// In Rust version: Validates and parses structured JSON response from AI
    pub async fn execute_code(
        &self,
        code: &str,
        _params: &JsonValue,
    ) -> Result<ExecutionResult> {
        // Parse the AI response as JSON
        let response: JsonValue = self.parse_ai_response(code)?;

        // Extract and validate action
        let action_str = response["action"]
            .as_str()
            .context("Missing 'action' field in AI response")?;

        let action = DiagnosisAction::from_str(action_str)
            .context(format!("Invalid action: {}", action_str))?;

        // Extract payload
        let payload = response["payload"]
            .clone();

        if payload.is_null() {
            anyhow::bail!("Missing 'payload' field in AI response");
        }

        // Validate payload based on action
        self.validate_payload(&action, &payload)?;

        Ok(ExecutionResult { action, payload })
    }

    /// Parse AI response, handling various formats (raw JSON, markdown-wrapped, etc.)
    fn parse_ai_response(&self, response: &str) -> Result<JsonValue> {
        // Try direct JSON parse first
        if let Ok(json) = serde_json::from_str::<JsonValue>(response) {
            return Ok(json);
        }

        // Try extracting from Markdown code blocks
        if response.contains("```json") {
            let extracted = response
                .split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .context("Failed to extract JSON from markdown")?
                .trim();

            return serde_json::from_str(extracted)
                .context("Failed to parse JSON from markdown block");
        }

        if response.contains("```") {
            let extracted = response
                .split("```")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .context("Failed to extract JSON from code block")?
                .trim();

            return serde_json::from_str(extracted)
                .context("Failed to parse JSON from code block");
        }

        // Last resort: try to find JSON object in the response
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                let json_str = &response[start..=end];
                if let Ok(json) = serde_json::from_str::<JsonValue>(json_str) {
                    return Ok(json);
                }
            }
        }

        anyhow::bail!("Could not parse AI response as valid JSON")
    }

    /// Validate that the payload contains required fields for the action
    fn validate_payload(&self, action: &DiagnosisAction, payload: &JsonValue) -> Result<()> {
        match action {
            DiagnosisAction::GetPlantVitals => {
                // No specific validation needed for GET_PLANT_VITALS
                Ok(())
            }
            DiagnosisAction::LogState => {
                // LOG_STATE should have at least one field in payload
                if !payload.is_object() || payload.as_object().unwrap().is_empty() {
                    anyhow::bail!("LOG_STATE payload must be a non-empty object");
                }
                Ok(())
            }
            DiagnosisAction::AskUser => {
                // ASK_USER must have a "question" field
                payload["question"]
                    .as_str()
                    .context("ASK_USER payload must contain a 'question' string")?;
                Ok(())
            }
            DiagnosisAction::Conclude => {
                // CONCLUDE must have "finding" and "recommendation" fields
                payload["finding"]
                    .as_str()
                    .context("CONCLUDE payload must contain a 'finding' string")?;
                payload["recommendation"]
                    .as_str()
                    .context("CONCLUDE payload must contain a 'recommendation' string")?;
                Ok(())
            }
        }
    }

    /// Execute a specific action with its payload
    /// This is called by the diagnosis service after validation
    pub fn execute_action(
        &self,
        result: &ExecutionResult,
        context: &mut JsonValue,
    ) -> Result<ActionEffect> {
        match result.action {
            DiagnosisAction::GetPlantVitals => {
                // Signal that plant vitals should be fetched
                Ok(ActionEffect::FetchPlantVitals)
            }
            DiagnosisAction::LogState => {
                // Update the state in the diagnosis context
                if let Some(state) = context.get_mut("state") {
                    if let Some(state_obj) = state.as_object_mut() {
                        if let Some(payload_obj) = result.payload.as_object() {
                            for (key, value) in payload_obj {
                                state_obj.insert(key.clone(), value.clone());
                            }
                        }
                    }
                }
                Ok(ActionEffect::Continue)
            }
            DiagnosisAction::AskUser => {
                let question = result.payload["question"]
                    .as_str()
                    .unwrap()
                    .to_string();
                Ok(ActionEffect::AskUser(question))
            }
            DiagnosisAction::Conclude => {
                let finding = result.payload["finding"]
                    .as_str()
                    .unwrap()
                    .to_string();
                let recommendation = result.payload["recommendation"]
                    .as_str()
                    .unwrap()
                    .to_string();
                Ok(ActionEffect::Conclude { finding, recommendation })
            }
        }
    }
}

/// The effect of executing an action
#[derive(Debug)]
pub enum ActionEffect {
    /// Continue processing (e.g., after LOG_STATE)
    Continue,
    /// Need to fetch plant vitals from database
    FetchPlantVitals,
    /// Ask user a question
    AskUser(String),
    /// Conclude the diagnosis
    Conclude {
        finding: String,
        recommendation: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_json_response() {
        let executor = SandboxExecutor::new();

        let json_str = r#"{"action": "ASK_USER", "payload": {"question": "Test?"}}"#;
        let result = executor.parse_ai_response(json_str).unwrap();

        assert_eq!(result["action"], "ASK_USER");
    }

    #[tokio::test]
    async fn test_parse_markdown_wrapped_json() {
        let executor = SandboxExecutor::new();

        let markdown = r#"
Here's the action:
```json
{"action": "CONCLUDE", "payload": {"finding": "Test", "recommendation": "Do this"}}
```
"#;
        let result = executor.parse_ai_response(markdown).unwrap();

        assert_eq!(result["action"], "CONCLUDE");
    }

    #[tokio::test]
    async fn test_validate_ask_user_payload() {
        let executor = SandboxExecutor::new();
        let payload = serde_json::json!({"question": "How often do you water?"});

        let result = executor.validate_payload(&DiagnosisAction::AskUser, &payload);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_conclude_payload() {
        let executor = SandboxExecutor::new();
        let payload = serde_json::json!({
            "finding": "Root rot",
            "recommendation": "Reduce watering"
        });

        let result = executor.validate_payload(&DiagnosisAction::Conclude, &payload);
        assert!(result.is_ok());
    }
}

