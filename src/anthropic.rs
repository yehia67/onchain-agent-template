use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::future::Future;
use std::pin::Pin;
use crate::personality::Personality;
use crate::tools::{execute_tool, get_available_tools};

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    system: Option<String>,
    messages: Vec<Message>,
    tools: Option<Vec<AnthropicTool>>,
}

#[derive(Serialize, Clone)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Serialize, Clone)]
pub struct Message {
    role: String,
    content: Vec<ContentBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<AnthropicToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Serialize, Clone)]
struct AnthropicToolCall {
    id: String,
    name: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

#[derive(Deserialize, Debug)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
    #[serde(default)]
    tool_calls: Vec<AnthropicToolCallResponse>,
   
 
}

#[derive(Deserialize, Debug)]
struct AnthropicToolCallResponse {
    id: String,
    name: String,
    parameters: serde_json::Value,
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




pub async fn call_anthropic_with_personality(prompt: &str, personality: Option<&Personality>) -> anyhow::Result<String> {
    // Check if this is a direct ETH send command before passing to Claude
    if prompt.to_lowercase().starts_with("send") && prompt.contains("ETH") {
        // This looks like an ETH send command, try to execute it directly
        let args = serde_json::json!({
            "operation": "send",
            "raw_command": prompt
        });
        
        match crate::tools::execute_tool("eth_wallet", &args).await {
            Ok(result) => return Ok(result),
            Err(e) => return Ok(format!("Error executing ETH transaction: {}", e)),
        }
    }
    
    // Otherwise, proceed with normal Claude processing
    call_anthropic_with_tools(prompt, personality, Vec::new()).await
}

pub fn call_anthropic_with_tools<'a>(
    prompt: &'a str, 
    personality: Option<&'a Personality>,
    previous_messages: Vec<Message>
) -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + 'a>> {
    Box::pin(async move {
    let api_key = env::var("ANTHROPIC_API_KEY")?;
    let client = Client::new();

    // Create messages vector
    let mut messages = previous_messages;
    
    // Create system prompt with personality if provided
    let mut system_prompt_parts = Vec::new();
    
    if let Some(persona) = personality {
        system_prompt_parts.push(format!(
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
        ));
    }
    
    // Add tool usage instructions to system prompt
    let tools = get_available_tools();
    if !tools.is_empty() {
        system_prompt_parts.push(format!(
            "\n\nYou have access to the following tools:\n{}\n\n\
            When you need to use a tool:\n\
            1. Respond with a tool call when a tool should be used\n\
            2. Wait for the tool response before providing your final answer\n\
            3. Don't fabricate tool responses - only use the actual results returned by the tool",
            tools.iter()
                .map(|t| format!("- {}: {}", t.name, t.description))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }
    
    let system_prompt = if !system_prompt_parts.is_empty() {
        Some(system_prompt_parts.join("\n\n"))
    } else {
        None
    };
    
    // Add user message if there are no previous messages or we need to add a new prompt
    if messages.is_empty() || !prompt.is_empty() {
        messages.push(Message {
            role: "user".to_string(),
            content: vec![ContentBlock::Text {
                text: prompt.to_string(),
            }],
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
    }
    
    // Convert tools to Anthropic format
    let anthropic_tools = if !tools.is_empty() {
        let mut anthropic_tools = Vec::new();
        
        for tool in tools {
            let input_schema = match tool.name.as_str() {
                "get_weather" => serde_json::json!({
                    "type": "object",
                    "properties": {
                        "city": {
                            "type": "string",
                            "description": "The city to get weather for"
                        }
                    },
                    "required": ["city"]
                }),
                "get_time" => serde_json::json!({
                    "type": "object",
                    "properties": {
                        "timezone": {
                            "type": "string",
                            "description": "Optional timezone (e.g., 'UTC', 'America/New_York'). If not provided, local time is returned."
                        }
                    }
                }),
                "eth_wallet" => serde_json::json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "description": "The operation to perform: 'generate', 'balance', or 'send'"
                        },
                        "address": {
                            "type": "string",
                            "description": "Ethereum address for 'balance' operation"
                        },
                        "from_address": {
                            "type": "string",
                            "description": "Sender's Ethereum address for 'send' operation"
                        },
                        "to_address": {
                            "type": "string",
                            "description": "Recipient's Ethereum address for 'send' operation"
                        },
                        "amount": {
                            "type": "string",
                            "description": "Amount of ETH to send for 'send' operation"
                        },
                        "private_key": {
                            "type": "string",
                            "description": "Private key for the sender's address (required for 'send' operation if the wallet is not stored)"
                        }
                    },
                    "required": ["operation"]
                }),
                _ => serde_json::json!({"type": "object", "properties": {}}),
            };
            
            anthropic_tools.push(AnthropicTool {
                name: tool.name,
                description: tool.description,
                input_schema,
            });
        }
        
        Some(anthropic_tools)
    } else {
        None
    };
    
    let req = AnthropicRequest {
        model: "claude-3-opus-20240229".to_string(),
        max_tokens: 1024,
        system: system_prompt,
        messages: messages.clone(), // Clone here to keep ownership
        tools: anthropic_tools,
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
    let response_data: AnthropicResponse = match serde_json::from_str(&response_text) {
        Ok(data) => data,
        Err(e) => {
            println!("Failed to parse response: {}", e);
            println!("Response text: {}", response_text);
            return Err(anyhow::anyhow!("Failed to parse Anthropic response: {}", e));
        }
    };

    // Check if there are tool calls in the response (either in tool_calls or content)
    let mut has_tool_call = false;
    let mut tool_name = String::new();
    let mut tool_id = String::new();
    let mut tool_parameters = serde_json::Value::Null;
    
    // First check for tool_use in content
    for content_block in &response_data.content {
        if let ContentBlock::ToolUse { id, name, input } = content_block {
            has_tool_call = true;
            tool_name = name.clone();
            tool_id = id.clone();
            tool_parameters = input.clone();
            break;
        }
    }
    
    // If no tool_use in content, check the tool_calls array (legacy format)
    if !has_tool_call && !response_data.tool_calls.is_empty() {
        has_tool_call = true;
        let tool_call = &response_data.tool_calls[0];
        tool_name = tool_call.name.clone();
        tool_id = tool_call.id.clone();
        tool_parameters = tool_call.parameters.clone();
    }
    
    if has_tool_call {
        // Execute the tool
        let tool_result = execute_tool(&tool_name, &tool_parameters).await?;
        
        // Create a tool response message with tool_use content
        let tool_response_message = Message {
            role: "assistant".to_string(),
            content: vec![ContentBlock::ToolUse {
                id: tool_id.clone(),
                name: tool_name.clone(),
                input: tool_parameters.clone(),
            }],
            tool_calls: None,
            tool_call_id: None,
            name: None,
        };
        
        // Add the tool response message to the conversation
        let mut new_messages = messages.clone();
        new_messages.push(tool_response_message);
        
        // Add the tool result message as a user message with tool_result content
        new_messages.push(Message {
            role: "user".to_string(),
            content: vec![ContentBlock::ToolResult {
                tool_use_id: tool_id.clone(),
                content: tool_result,
            }],
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
        
        // Call the API again with the tool result
        return call_anthropic_with_tools("", personality, new_messages).await;
    }
    
    // If no tool calls, return the text response
    let response_text = response_data.content.iter()
        .filter_map(|block| {
            match block {
                ContentBlock::Text { text } => Some(text.clone()),
                _ => None,
            }
        })
        .collect::<Vec<String>>()
        .join("");
        
    // If the response is empty, add a fallback message
    let response_text = if response_text.trim().is_empty() {
        "I'm processing your request...".to_string()
    } else {
        response_text
    };

    Ok(response_text)
    })
}
