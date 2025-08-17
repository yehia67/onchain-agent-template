use serde::{Deserialize, Serialize};
use chrono::Local;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub action: String, // "call_tool" or "answer"
    pub tool: Option<String>,
    pub args: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResponse {
    pub tool: String,
    pub content: String,
}

pub fn get_available_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "get_weather".to_string(),
            description: "Get the current weather for a given city".to_string(),
        },
        Tool {
            name: "get_time".to_string(),
            description: "Get the current time in a specific timezone or local time".to_string(),
        },
    ]
}

pub fn get_tools_as_json() -> anyhow::Result<String> {
    let tools = get_available_tools();
    Ok(serde_json::to_string_pretty(&tools)?)
}

pub async fn execute_tool(name: &str, args: &serde_json::Value) -> anyhow::Result<String> {
    match name {
        "get_weather" => {
            let city = args.get("city")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            
            get_weather(city).await
        },
        "get_time" => {
            let timezone = args.get("timezone")
                .and_then(|v| v.as_str());
            
            get_time(timezone)
        },
        _ => Ok(format!("Unknown tool: {}", name)),
    }
}

async fn get_weather(city: &str) -> anyhow::Result<String> {
    // In a real implementation, you would call a weather API
    // For this example, we'll return mock data
    
    // Simulate API call delay
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Return mock data
    match city.to_lowercase().as_str() {
        "cairo" => Ok("30°C, sunny".to_string()),
        "london" => Ok("15°C, cloudy with occasional rain".to_string()),
        "new york" => Ok("22°C, partly cloudy".to_string()),
        "tokyo" => Ok("25°C, clear skies".to_string()),
        _ => Ok(format!("Weather data for {} is not available. This is a mock implementation.", city)),
    }
}

fn get_time(timezone: Option<&str>) -> anyhow::Result<String> {
    let now = Local::now();
    
    match timezone {
        Some(tz) => {
            // In a real implementation, you would handle different timezones
            // For this example, we'll just return the local time with a note
            Ok(format!("Current time (local, timezone {} not implemented): {}", 
                      tz, now.format("%Y-%m-%d %H:%M:%S")))
        },
        None => {
            Ok(format!("Current local time: {}", now.format("%Y-%m-%d %H:%M:%S")))
        }
    }
}
