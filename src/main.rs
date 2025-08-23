mod anthropic;
mod db;
mod personality;
mod tools;

use db::{get_db_pool, save_message};
use anthropic::call_anthropic_with_personality;
use personality::load_personality;
use tools::get_tools_as_json;
use std::io::{self, Write};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let pool = get_db_pool().await;
    
    // Load personality
    let personality_path = Path::new("assets/personality.json");
    let personality = match load_personality(personality_path.to_str().unwrap()) {
        Ok(p) => {
            println!("Loaded personality: {} - {}", p.name, p.role);
            p
        },
        Err(e) => {
            println!("Failed to load personality: {}", e);
            return Err(anyhow::anyhow!("Failed to load personality"));
        }
    };
    
    // Load available tools
    match get_tools_as_json() {
        Ok(tools_json) => {
            println!("Loaded tools: {}", tools_json);
        },
        Err(e) => {
            println!("Failed to load tools: {}", e);
        }
    };
    
    println!("Welcome to Agent Friend! I'm {}, your {}.", personality.name, personality.role);
    println!("Type 'exit' to quit.");
    
    loop {
        // Prompt for user input
        print!("You: ");
        io::stdout().flush()?;
        
        // Read user input
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();
        
        // Check if user wants to exit
        if user_input.to_lowercase() == "exit" {
            println!("Goodbye!");
            break;
        }
        
        // Skip empty inputs
        if user_input.is_empty() {
            continue;
        }
        
        // Save user message to database if pool is available
        if let Some(pool) = &pool {
            if let Err(e) = save_message(pool, "user", user_input).await {
                eprintln!("Failed to save user message: {}", e);
            }
        }
        
        // Get response from Claude with personality
        print!("{} is thinking...", personality.name);
        io::stdout().flush()?;
        let reply = call_anthropic_with_personality(user_input, Some(&personality)).await?;
        println!("\r"); // Clear the "thinking" message
        
        // Save assistant message to database if pool is available
        if let Some(pool) = &pool {
            if let Err(e) = save_message(pool, "assistant", &reply).await {
                eprintln!("Failed to save assistant message: {}", e);
            }
        }
        
        // Display the response
        println!("{}: {}", personality.name, reply);
    }
    
    Ok(())
}
