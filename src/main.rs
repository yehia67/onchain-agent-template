mod anthropic;
mod db;

use db::{get_db_pool, save_message};
use anthropic::call_anthropic;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let pool = get_db_pool().await;
    
    println!("Welcome to Agent Friend! Type 'exit' to quit.");
    
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
        
        // Save user message to database
        save_message(&pool, "user", user_input).await?;
        
        // Get response from Claude
        print!("Agent is thinking...");
        io::stdout().flush()?;
        let reply = call_anthropic(user_input).await?;
        println!("\r"); // Clear the "thinking" message
        
        // Save assistant message to database
        save_message(&pool, "assistant", &reply).await?;
        
        // Display the response
        println!("Agent: {}", reply);
    }
    
    Ok(())
}
