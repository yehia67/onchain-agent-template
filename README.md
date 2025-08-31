# Onchain AI Agent Template

A Rust-based AI agent template that can perform both on-chain and off-chain operations, built with Tokio and Anthropic's Claude API.

## Features

- 🤖 Basic chat interface with Claude AI
- 🎭 Customizable agent personality
- 💾 PostgreSQL database integration for message history
- 🛠️ Tool integration for external actions
- ⛓️ Ethereum blockchain integration (wallet generation, balance checks, transactions)

## Architecture

The project is structured into several key modules:

```
agent-friend/
├── src/
│   ├── main.rs          # Entry point and main loop
│   ├── anthropic.rs     # Claude API integration
│   ├── personality.rs   # Personality customization
│   ├── db.rs            # Database operations
│   ├── tools.rs         # Tool implementations
│   └── bin/             # Additional binaries
├── assets/
│   └── personality.json # Agent personality configuration
├── migrations/
│   └── *.sql            # Database migration files
├── .env.example         # Example environment variables
└── Cargo.toml           # Project dependencies
```

### Core Components

1. **Main Loop** (`main.rs`): Handles user input/output and orchestrates the agent's components
2. **Anthropic Integration** (`anthropic.rs`): Manages communication with Claude API
3. **Personality System** (`personality.rs`): Loads and applies personality traits to the agent
4. **Database Layer** (`db.rs`): Stores conversation history in PostgreSQL
5. **Tools System** (`tools.rs`): Implements external functionalities like weather info and Ethereum operations

## Prerequisites

- Rust and Cargo
- PostgreSQL database
- Anthropic API key
- Ethereum RPC URL (for blockchain features)

## Setup Instructions

### 1. Clone the repository

```bash
git clone https://github.com/yehia67/onchain-agent-template.git
cd onchain-agent-template
```

### 2. Set up environment variables

Copy the example environment file and add your credentials:

```bash
cp .env.example .env
```

Edit the `.env` file to include:
- Your Anthropic API key (get it from [Anthropic Console](https://console.anthropic.com/settings/keys))
- PostgreSQL database connection string
- Ethereum RPC URL (e.g., Sepolia testnet)

### 3. Set up the database

Create a PostgreSQL database and user:

```bash
# Example commands - adjust as needed for your PostgreSQL setup
createdb agentdb
createuser -P agent  # Set password to 'agent' when prompted
```

Run database migrations:

```bash
# If you have sqlx-cli installed
sqlx migrate run

# Alternatively, the migrations will run automatically on first startup
```

### 4. Build and run the project

```bash
cargo build
cargo run
```

## Usage

Once running, you can interact with the agent via the command line:

- Type messages and press Enter to send them to the agent
- The agent will respond based on its personality and capabilities
- Use natural language to request actions like "What's the weather in Tokyo?" or "Generate a new Ethereum wallet"
- Type 'exit' to quit

## Ethereum Features

The agent can:
- Generate new Ethereum wallets
- Check ETH balances
- Send ETH transactions (on Sepolia testnet by default)

Example commands:
- "Generate a new Ethereum wallet"
- "Check the balance of 0x123..."
- "Send 0.1 ETH from 0x123... to 0x456..."

## Extending the Agent

You can extend this template by:
- Adding new tools in `tools.rs`
- Modifying the personality in `assets/personality.json`
- Adding more blockchain capabilities
- Creating a web or mobile interface

## License

MIT
