use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use crate::personality::Personality;

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    system: Option<String>,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: Vec<ContentBlock>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ContentBlock {
    #[serde(rename = "type")]
    r#type: String,
    text: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct AnthropicResponse {
    id: String,
    model: String,
    role: String,
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: Option<Usage>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct AnthropicErrorResponse {
    #[serde(rename = "type")]
    error_type: String,
    error: AnthropicError,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct AnthropicError {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

pub async fn call_anthropic(prompt: &str) -> anyhow::Result<String> {
    // Use default system prompt
    call_anthropic_with_personality(prompt, None).await
}

pub async fn call_anthropic_with_personality(prompt: &str, personality: Option<&Personality>) -> anyhow::Result<String> {
    let api_key = env::var("ANTHROPIC_API_KEY")?;
    let client = Client::new();

    // Create messages vector
    let mut messages = Vec::new();
    
    // Create system prompt with personality if provided
    let system_prompt = if let Some(persona) = personality {
        Some(format!(
            "You are {}, {}. \n\n\
            Style: \n\
            - Tone: {} \n\
            - Formality: {} \n\
            - Domain Focus: {} \n\n\
            Rules: \n{}",
            persona.name,
            persona.role,
            persona.style.tone,
            persona.style.formality,
            persona.style.domain_focus.join(", "),
            persona.rules.iter().map(|r| format!("- {}", r)).collect::<Vec<_>>().join("\n")
        ))
    } else {
        None
    };
    
    // Add user message
    messages.push(Message {
        role: "user".to_string(),
        content: vec![ContentBlock {
            r#type: "text".to_string(),
            text: prompt.to_string(),
        }],
    });
    
    let req = AnthropicRequest {
        model: "claude-3-opus-20240229".to_string(),
        max_tokens: 256,
        system: system_prompt,
        messages,
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&req)
        .send()
        .await?;
        
    // Get the response text
    let response_text = response.text().await?;
    
    // Try to parse as error response first
    if let Ok(error_response) = serde_json::from_str::<AnthropicErrorResponse>(&response_text) {
        return Err(anyhow::anyhow!("Anthropic API error: {}: {}", 
            error_response.error.error_type, 
            error_response.error.message));
    }
    
    // If not an error, parse as successful response
    let res: AnthropicResponse = serde_json::from_str(&response_text)?;

    Ok(res
        .content
        .get(0)
        .map(|c| c.text.clone())
        .unwrap_or_else(|| "No response".to_string()))
}
