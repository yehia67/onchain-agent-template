use serde::{Deserialize, Serialize};
use chrono::Local;
use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use rand::Rng;
use std::str::FromStr;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use std::env;

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
        Tool {
            name: "eth_wallet".to_string(),
            description: "Ethereum wallet operations: generate new wallet, check balance, or send ETH".to_string(),
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
        "eth_wallet" => {
            let operation = args.get("operation")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            
            match operation {
                "generate" => {
                    eth_generate_wallet().await
                },
                "balance" => {
                    let address = args.get("address")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    
                    eth_check_balance(address).await
                },
                "send" => {
                    // Check if we have a raw command string in the args
                    if let Some(raw_command) = args.get("raw_command").and_then(|v| v.as_str()) {
                        // Try to parse the natural language command
                        return parse_and_execute_eth_send_command(raw_command).await;
                    }
                    
                    // Otherwise use the structured parameters
                    let from_address = args.get("from_address")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let to_address = args.get("to_address")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let amount = args.get("amount")
                        .and_then(|v| v.as_str())
                        .unwrap_or("0");
                    let private_key = args.get("private_key")
                        .and_then(|v| v.as_str());
                    
                    eth_send_eth(from_address, to_address, amount, private_key).await
                },
                _ => Ok(format!("Unknown Ethereum wallet operation: {}", operation)),
            }
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

// In-memory wallet storage (for demo purposes)
lazy_static::lazy_static! {
    static ref WALLETS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

// Sepolia RPC URL
fn get_sepolia_rpc_url() -> String {
    env::var("SEPOLIA_RPC_URL")
        .expect("SEPOLIA_RPC_URL must be set")
}

// Get provider for Ethereum network
async fn get_provider() -> anyhow::Result<Provider<Http>> {
    // Use environment variable if available, otherwise use default
    let rpc_url = env::var("ETH_RPC_URL").unwrap_or_else(|_| get_sepolia_rpc_url());
    
    // Create provider
    let provider = Provider::<Http>::try_from(rpc_url)?;
    Ok(provider)
}

// Ethereum wallet functions
async fn eth_generate_wallet() -> anyhow::Result<String> {
    // Generate a new random private key
    let mut rng = rand::thread_rng();
    let mut private_key_bytes: [u8; 32] = [0; 32];
    rng.fill(&mut private_key_bytes);
    let private_key = hex::encode(&private_key_bytes);
    
    // Create wallet from private key
    let wallet = match LocalWallet::from_bytes(&private_key_bytes) {
        Ok(wallet) => wallet,
        Err(_) => return Ok("Failed to generate wallet".to_string()),
    };
    
    // Get the wallet address
    let address = wallet.address();
    
    // Store the private key and address pair (for demo purposes)
    let mut wallets = WALLETS.lock().unwrap();
    wallets.insert(format!("{:?}", address), private_key.clone());
    
    Ok(format!("Generated new Ethereum wallet:\nAddress: {:?}\nPrivate Key: {}", address, private_key))
}

async fn eth_check_balance(address: &str) -> anyhow::Result<String> {
    if address.is_empty() {
        return Ok("Error: Address is required".to_string());
    }
    
    // Parse the address
    let address_result = Address::from_str(address);
    let address = match address_result {
        Ok(addr) => addr,
        Err(_) => return Ok(format!("Error: Invalid Ethereum address format: {}", address)),
    };
    
    // Get provider
    let provider = match get_provider().await {
        Ok(provider) => provider,
        Err(e) => return Ok(format!("Error connecting to Ethereum node: {}", e)),
    };
    
    // Get balance from the network
    match provider.get_balance(address, None).await {
        Ok(balance) => {
            // Convert from Wei to ETH (1 ETH = 10^18 Wei)
            let eth_balance = balance.as_u128() as f64 / 1_000_000_000_000_000_000.0;
            Ok(format!("Balance for address {:?}: {:.6} ETH (via {})", 
                      address, eth_balance, get_sepolia_rpc_url()))
        },
        Err(e) => {
            // Fallback to mock data if there's an error
            println!("Error fetching balance, using mock data: {}", e);
            let mock_balance = format!("{}.{} ETH (mock)", 
                                     rand::thread_rng().gen_range(0..10), 
                                     rand::thread_rng().gen_range(100000..999999));
            Ok(format!("Balance for address {:?}: {}", address, mock_balance))
        }
    }
}

// Parse and execute a natural language ETH send command
async fn parse_and_execute_eth_send_command(command: &str) -> anyhow::Result<String> {
    println!("Parsing ETH send command: {}", command);
    
    // Extract amount (look for pattern like "0.1 ETH" or "0.1ETH")
    let amount_pattern = regex::Regex::new(r"(\d+\.?\d*) ?ETH").unwrap();
    let amount = match amount_pattern.captures(command) {
        Some(caps) => caps.get(1).map_or("", |m| m.as_str()),
        None => return Ok("Error: Could not parse ETH amount from command".to_string()),
    };
    
    // Extract from_address (look for pattern like "from 0x...")
    let from_pattern = regex::Regex::new(r"from (0x[a-fA-F0-9]{40})").unwrap();
    let from_address = match from_pattern.captures(command) {
        Some(caps) => caps.get(1).map_or("", |m| m.as_str()),
        None => return Ok("Error: Could not parse from address from command".to_string()),
    };
    
    // Extract to_address (look for pattern like "to 0x...")
    let to_pattern = regex::Regex::new(r"to (0x[a-fA-F0-9]{40})").unwrap();
    let to_address = match to_pattern.captures(command) {
        Some(caps) => caps.get(1).map_or("", |m| m.as_str()),
        None => return Ok("Error: Could not parse to address from command".to_string()),
    };
    
    // Extract private key (look for pattern like "private key ...")
    let key_pattern = regex::Regex::new(r"private key ([a-fA-F0-9]{64})").unwrap();
    let private_key = key_pattern.captures(command).map(|caps| caps.get(1).map_or("", |m| m.as_str()));
    
    println!("Parsed command - From: {}, To: {}, Amount: {}, Has Private Key: {}", 
             from_address, to_address, amount, private_key.is_some());
    
    // Execute the transaction with the parsed parameters
    eth_send_eth(from_address, to_address, amount, private_key).await
}

async fn eth_send_eth(from_address: &str, to_address: &str, amount: &str, provided_private_key: Option<&str>) -> anyhow::Result<String> {
    if from_address.is_empty() || to_address.is_empty() || amount.is_empty() {
        return Ok("Error: From address, to address, and amount are required".to_string());
    }
    
    // Parse the addresses
    let from_address_result = Address::from_str(from_address);
    let from_address = match from_address_result {
        Ok(addr) => addr,
        Err(_) => return Ok(format!("Error: Invalid from address format: {}", from_address)),
    };
    
    let to_address_result = Address::from_str(to_address);
    let to_address = match to_address_result {
        Ok(addr) => addr,
        Err(_) => return Ok(format!("Error: Invalid to address format: {}", to_address)),
    };
    
    // Parse amount
    let amount_eth = match amount.parse::<f64>() {
        Ok(val) => val,
        Err(_) => return Ok(format!("Error: Invalid amount: {}", amount)),
    };
    
    // Get the private key - either from the provided parameter or from stored wallets
    let private_key = if let Some(key) = provided_private_key {
        // Use the provided private key
        key.to_string()
    } else {
        // Check if we have the private key for this address in our wallet storage
        let wallets = WALLETS.lock().unwrap();
        match wallets.get(&format!("{:?}", from_address)) {
            Some(key) => key.clone(),
            None => {
                return Ok(format!("Error: No private key found for address {:?}. Please provide a private key.", from_address))
            }
        }
    };
    // No need to hold the lock anymore if we accessed the wallets
    let private_key_bytes = match hex::decode(&private_key) {
        Ok(bytes) => bytes,
        Err(_) => return Ok("Error: Invalid private key format".to_string()),
    };
    
    // Get provider
    let provider = match get_provider().await {
        Ok(provider) => provider,
        Err(e) => return Ok(format!("Error connecting to Ethereum node: {}", e)),
    };
    
    // Create wallet from private key
    let wallet = match LocalWallet::from_bytes(&private_key_bytes) {
        Ok(wallet) => wallet.with_chain_id(11155111u64), // Sepolia chain ID
        Err(_) => return Ok("Error: Failed to create wallet from private key".to_string()),
    };
    
    // Create a client with the wallet
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);
    
    // Convert ETH amount to Wei (1 ETH = 10^18 Wei)
    let wei_amount = (amount_eth * 1_000_000_000_000_000_000.0) as u128;
    let wei_amount = U256::from(wei_amount);
    
    // Get current gas price
    let gas_price = match client.get_gas_price().await {
        Ok(price) => price,
        Err(e) => return Ok(format!("Error getting gas price: {}", e)),
    };
    
    // Create transaction request
    let tx = TransactionRequest::new()
        .to(to_address)
        .value(wei_amount)
        .from(from_address);
            
    // Convert TransactionRequest to TypedTransaction before estimating gas
    let typed_tx = TypedTransaction::Legacy(tx);
    
    // Estimate gas for the transaction
    let gas_estimate = match client.estimate_gas(&typed_tx, None).await {
        Ok(estimate) => estimate,
        Err(e) => return Ok(format!("Error estimating gas: {}", e)),
    };
    
    // Actually send the transaction
    match client.send_transaction(typed_tx, None).await {
        Ok(pending_tx) => {
            // Get the transaction hash immediately
            let tx_hash = pending_tx.tx_hash();
            
            // Try to get the transaction receipt with a timeout
            let receipt_future = pending_tx.confirmations(1);
            match tokio::time::timeout(std::time::Duration::from_secs(60), receipt_future).await {
                Ok(receipt_result) => {
                    match receipt_result {
                        Ok(receipt) => {
                            // Transaction was mined successfully
                            // The receipt is an Option<TransactionReceipt>, so we need to unwrap it first
                            if let Some(receipt_data) = receipt {
                                Ok(format!("Transaction successfully sent {} ETH from {:?} to {:?}\n\
                                          Gas Price: {} gwei\n\
                                          Gas Used: {}\n\
                                          Block Number: {}\n\
                                          Network: Sepolia (via {})\n\
                                          Transaction Hash: {:?}", 
                                          amount_eth, from_address, to_address, 
                                          gas_price.as_u128() / 1_000_000_000, // Convert to gwei
                                          receipt_data.gas_used.unwrap_or_default(),
                                          receipt_data.block_number.unwrap_or_default(),
                                          get_sepolia_rpc_url(),
                                          tx_hash))
                            } else {
                                // Transaction was submitted but no receipt was found
                                Ok(format!("Transaction submitted but no receipt was found.\n\
                                          {} ETH from {:?} to {:?}\n\
                                          Network: Sepolia (via {})\n\
                                          Transaction Hash: {:?}", 
                                          amount_eth, from_address, to_address,
                                          get_sepolia_rpc_url(),
                                          tx_hash))
                            }
                        },
                        Err(e) => {
                            // Transaction was submitted but failed during mining
                            Ok(format!("Transaction submitted but failed: {}\n\
                                      Transaction Hash: {:?}", e, tx_hash))
                        }
                    }
                },
                Err(_) => {
                    // Timeout waiting for transaction to be mined
                    // Return the transaction hash anyway since it was submitted
                    Ok(format!("Transaction submitted but confirmation timed out after 60 seconds.\n\
                              {} ETH from {:?} to {:?}\n\
                              Gas Price: {} gwei\n\
                              Gas Estimate: {}\n\
                              Network: Sepolia (via {})\n\
                              Transaction Hash: {:?}", 
                              amount_eth, from_address, to_address, 
                              gas_price.as_u128() / 1_000_000_000, // Convert to gwei
                              gas_estimate,
                              get_sepolia_rpc_url(),
                              tx_hash))
                }
            }
        },
        Err(e) => {
            // Failed to send transaction
            Ok(format!("Error sending transaction: {}", e))
        }
    }
}
