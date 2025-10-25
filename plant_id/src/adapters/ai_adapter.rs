//! AI ADAPTER
//!
//! Secondary adapter for interacting with AI models via OpenRouter API.
//! Handles chat completions and care schedule generation.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::get_env;
use crate::domain::CareSchedule;

#[derive(Clone)]
pub struct AiAdapter {
    client: Client,
    api_key: String,
    model: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

impl AiAdapter {
    pub fn new() -> Result<Self> {
        let api_key = get_env("OPENROUTER_API_KEY")?;
        let model = std::env::var("AI_MODEL")
            .unwrap_or_else(|_| "anthropic/claude-3.5-sonnet".to_string());

        Ok(Self {
            client: Client::new(),
            api_key,
            model,
        })
    }

    pub async fn get_completion(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let request = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": user_prompt
                }
            ]
        });

        let response = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("AI API error: {}", error_text);
        }

        let completion: ChatCompletionResponse = response.json().await?;

        completion
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .context("No response from AI")
    }

    pub async fn generate_care_schedule(&self, plant_name: &str) -> Result<CareSchedule> {
        // Using the prompt you provided
        let system_prompt = r#"You are an expert Botanist. The user will provide you with the name of a plant.
Your task is to research this plant and provide a detailed care schedule.
You MUST return your response as a single, minified JSON object with NO markdown formatting.
The JSON object must have the following fields:
{
  "light": "description of light requirements",
  "water": "description of watering schedule",
  "humidity": "description of humidity requirements",
  "temperature": "description of temperature range",
  "care_instructions": "additional care tips and notes"
}
Be specific and practical in your recommendations."#;

        let user_prompt = format!("Generate a care schedule for: {}", plant_name);

        let response = self.get_completion(system_prompt, &user_prompt).await?;

        // Extract JSON from response (may be wrapped in markdown code blocks)
        let json_str = if response.contains("```json") {
            response
                .split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(&response)
                .trim()
        } else if response.contains("```") {
            response
                .split("```")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(&response)
                .trim()
        } else {
            response.trim()
        };

        let care_schedule: CareSchedule = serde_json::from_str(json_str)
            .context("Failed to parse care schedule from AI response")?;

        Ok(care_schedule)
    }

    pub async fn generate_diagnosis_response(&self, diagnosis_context: &serde_json::Value) -> Result<String> {
        // Using the simplified diagnostic kernel prompt for JSON responses
        let system_prompt = r#"You are a plant diagnostic AI. Your job is to analyze plant problems and determine the next action.

Analyze the diagnosis context and return a JSON response with "action" and "payload" keys.

Available Actions:
1. GET_PLANT_VITALS: Fetch plant data (use if plant_vitals is null)
   {"action": "GET_PLANT_VITALS", "payload": {}}

2. LOG_STATE: Store intermediate findings
   {"action": "LOG_STATE", "payload": {"hypothesis": "sun scorch", "confidence": 0.7}}

3. ASK_USER: Ask a clarifying question
   {"action": "ASK_USER", "payload": {"question": "How many hours of direct sunlight does your plant get?"}}

4. CONCLUDE: Provide final diagnosis
   {"action": "CONCLUDE", "payload": {"finding": "Sun Scorch", "recommendation": "Move to bright, indirect light"}}

Strategy:
1. Check if plant_vitals is null - if so, use GET_PLANT_VITALS
2. Ask 2-4 targeted questions to narrow down the issue
3. Track hypotheses using LOG_STATE
4. When confident, use CONCLUDE

Return ONLY valid JSON, no markdown formatting."#;

        let user_prompt = format!(
            "Analyze this diagnosis context and determine the next action:\n\n{}",
            serde_json::to_string_pretty(diagnosis_context)?
        );

        let response = self.get_completion(system_prompt, &user_prompt).await?;

        Ok(response)
    }
}