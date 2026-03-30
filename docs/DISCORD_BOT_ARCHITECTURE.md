# Discord Bot Architecture

Visual diagrams and architecture documentation for the Discord bot integration.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Discord Platform                            │
│                                                                     │
│  User types: /test-swap from:ethereum_sepolia:eth to:base:wbtc    │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             │ HTTPS (WebSocket)
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Your Server / Machine                            │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │                    Discord Bot Process                        │ │
│  │                 (cargo run -- discord-bot)                    │ │
│  │                                                               │ │
│  │  ┌─────────────────────────────────────────────────────────┐ │ │
│  │  │  Serenity (Discord Gateway)                             │ │ │
│  │  │  - WebSocket connection to Discord                      │ │ │
│  │  │  - Receives slash command events                        │ │ │
│  │  │  - Sends responses back                                 │ │ │
│  │  └────────────────────┬────────────────────────────────────┘ │ │
│  │                       │                                       │ │
│  │                       ▼                                       │ │
│  │  ┌─────────────────────────────────────────────────────────┐ │ │
│  │  │  Poise (Command Framework)                              │ │ │
│  │  │  - Parses slash commands                                │ │ │
│  │  │  - Routes to command handlers                           │ │ │
│  │  │  - Manages context and state                            │ │ │
│  │  └────────────────────┬────────────────────────────────────┘ │ │
│  │                       │                                       │ │
│  │                       ▼                                       │ │
│  │  ┌─────────────────────────────────────────────────────────┐ │ │
│  │  │  Command Handlers (src/discord/commands/)               │ │ │
│  │  │                                                          │ │ │
│  │  │  ping()         - Health check                          │ │ │
│  │  │  test_swap()    - Single swap test                      │ │ │
│  │  │  test_swap_all()- Batch swap test                       │ │ │
│  │  └────────────────────┬────────────────────────────────────┘ │ │
│  │                       │                                       │ │
│  │                       │ ctx.defer()                           │ │ │
│  │                       │ (prevent timeout)                     │ │ │
│  │                       │                                       │ │ │
│  │                       ▼                                       │ │
│  │  ┌─────────────────────────────────────────────────────────┐ │ │
│  │  │  tokio::process::Command                                │ │ │
│  │  │  - Spawns cargo subprocess                              │ │ │
│  │  │  - Non-blocking async execution                         │ │ │
│  │  │  - Captures stdout and stderr                           │ │ │
│  │  └────────────────────┬────────────────────────────────────┘ │ │
│  └───────────────────────┼───────────────────────────────────────┘ │
│                          │                                         │
│                          ▼                                         │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │  Cargo Subprocess                                           │  │
│  │  cargo run --release -- test-swap ethereum_sepolia:eth ... │  │
│  │                                                             │  │
│  │  ┌───────────────────────────────────────────────────────┐ │  │
│  │  │  Your Existing CLI Application                        │ │  │
│  │  │  - SwapRunner                                          │ │  │
│  │  │  - Garden API Client                                   │ │  │
│  │  │  - Chain Signers                                       │ │  │
│  │  │  - Database                                            │ │  │
│  │  └───────────────────────────────────────────────────────┘ │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                          │                                         │
│                          │ stdout/stderr                           │
│                          │                                         │
│                          ▼                                         │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │  Output Capture & Formatting                                │  │
│  │  - Concatenate stdout + stderr                              │  │
│  │  - Format with markdown                                     │  │
│  │  - Handle 2000 char limit                                   │  │
│  └────────────────────────┬────────────────────────────────────┘  │
└─────────────────────────────┼──────────────────────────────────────┘
                              │
                              │ Response
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Discord Platform                            │
│                                                                     │
│  Bot replies with:                                                 │
│  ✅ Command completed successfully                                 │
│  [Full output with transaction hashes and status]                 │
└─────────────────────────────────────────────────────────────────────┘
```

## Module Structure

```
src/
├── main.rs                          # Entry point
│   ├── mod discord;                 # Import Discord module
│   └── match "discord-bot" => ...   # Start bot
│
└── discord/                         # Discord bot module
    ├── mod.rs                       # Module exports
    │   ├── pub mod bot;
    │   ├── pub mod commands;
    │   └── pub use bot::start_discord_bot;
    │
    ├── bot.rs                       # Bot initialization
    │   ├── struct Data {}           # Shared state
    │   ├── type Context<'a>         # Command context
    │   └── async fn start_discord_bot()
    │       ├── Create framework
    │       ├── Register commands
    │       ├── Create client
    │       └── Start event loop
    │
    └── commands/                    # Command implementations
        ├── mod.rs                   # Command exports
        │   └── pub use test_swap::*;
        │
        └── test_swap.rs             # Slash commands
            ├── async fn ping()
            ├── async fn test_swap()
            ├── async fn test_swap_all()
            ├── async fn send_output()
            └── fn split_into_chunks()
```

## Data Flow

### 1. Command Reception

```
Discord User
    │
    │ Types: /test-swap from:ethereum_sepolia:eth to:base_sepolia:wbtc
    │
    ▼
Discord API
    │
    │ WebSocket Event
    │
    ▼
Serenity Gateway
    │
    │ Parse Event
    │
    ▼
Poise Framework
    │
    │ Route to Handler
    │
    ▼
test_swap() function
```

### 2. Command Execution

```
test_swap() function
    │
    │ 1. ctx.defer() - Tell Discord we're working
    │
    ▼
    │ 2. Get project root directory
    │
    ▼
    │ 3. Build command args
    │    ["run", "--release", "--", "test-swap", "ethereum_sepolia:eth", "base_sepolia:wbtc"]
    │
    ▼
tokio::process::Command
    │
    │ 4. Spawn subprocess
    │    .current_dir(project_root)
    │    .stdout(piped)
    │    .stderr(piped)
    │
    ▼
Cargo Process
    │
    │ 5. Execute CLI command
    │    cargo run --release -- test-swap ethereum_sepolia:eth base_sepolia:wbtc
    │
    ▼
Your CLI Application
    │
    │ 6. Run swap test
    │    - Create quote
    │    - Submit order
    │    - Sign transaction
    │    - Poll status
    │
    ▼
Output Streams
    │
    │ 7. Capture output
    │    stdout: "═══ Swap Test Result ═══\nPair: ...\nStatus: Completed\n..."
    │    stderr: (any errors or warnings)
    │
    ▼
test_swap() function
```

### 3. Response Formatting

```
test_swap() function
    │
    │ 8. Concatenate stdout + stderr
    │
    ▼
    │ 9. Format with markdown
    │    **STDOUT:**
    │    ```
    │    [output]
    │    ```
    │
    ▼
    │ 10. Add status indicator
    │     ✅ Command completed successfully (exit code: 0)
    │
    ▼
send_output() function
    │
    │ 11. Check output length
    │
    ├─▶ < 2000 chars ──▶ Single message
    │
    ├─▶ 2000-10000 chars ──▶ Split into multiple messages
    │
    └─▶ > 10000 chars ──▶ Send as file attachment
    │
    ▼
Discord API
    │
    │ 12. Post response
    │
    ▼
Discord User
    │
    │ 13. See result in Discord
    │
    ✓
```

## Sequence Diagram

```
User          Discord       Bot         Poise      Command      Subprocess     CLI App
 │              │            │            │           │             │            │
 │─/test-swap──▶│            │            │           │             │            │
 │              │            │            │           │             │            │
 │              │─Event─────▶│            │           │             │            │
 │              │            │            │           │             │            │
 │              │            │─Parse─────▶│           │             │            │
 │              │            │            │           │             │            │
 │              │            │            │─Route────▶│             │            │
 │              │            │            │           │             │            │
 │              │            │            │           │─defer()────▶│            │
 │              │◀───────────────────────────────────────────────────            │
 │              │            │            │           │             │            │
 │◀─Thinking────│            │            │           │             │            │
 │              │            │            │           │             │            │
 │              │            │            │           │─spawn()────▶│            │
 │              │            │            │           │             │            │
 │              │            │            │           │             │─execute───▶│
 │              │            │            │           │             │            │
 │              │            │            │           │             │            │─┐
 │              │            │            │           │             │            │ │ Run
 │              │            │            │           │             │            │ │ Swap
 │              │            │            │           │             │            │ │ Test
 │              │            │            │           │             │            │◀┘
 │              │            │            │           │             │            │
 │              │            │            │           │             │◀─output────│
 │              │            │            │           │             │            │
 │              │            │            │           │◀─stdout/────│            │
 │              │            │            │           │   stderr    │            │
 │              │            │            │           │             │            │
 │              │            │            │           │─format()────┐            │
 │              │            │            │           │             │            │
 │              │            │            │           │◀────────────┘            │
 │              │            │            │           │             │            │
 │              │            │            │◀─response─│             │            │
 │              │            │            │           │             │            │
 │              │            │◀───────────│           │             │            │
 │              │            │            │           │             │            │
 │              │◀─message───│            │           │             │            │
 │              │            │            │           │             │            │
 │◀─Result──────│            │           │             │            │
 │              │            │            │           │             │            │
```

## Component Responsibilities

### Serenity (Discord Gateway)
- Maintains WebSocket connection to Discord
- Handles authentication and heartbeat
- Receives events from Discord
- Sends messages back to Discord
- Manages rate limiting

### Poise (Command Framework)
- Parses slash command syntax
- Validates command parameters
- Routes commands to handlers
- Manages command context
- Handles command registration
- Provides ctx.defer() for long operations

### Command Handlers
- Implement business logic for each command
- Call ctx.defer() to prevent timeouts
- Spawn subprocesses for CLI commands
- Capture and format output
- Handle errors gracefully
- Send responses back to Discord

### tokio::process::Command
- Spawns cargo subprocess asynchronously
- Sets working directory to project root
- Pipes stdout and stderr
- Non-blocking execution
- Integrates with tokio runtime

### Output Formatter
- Concatenates stdout and stderr
- Adds markdown formatting
- Checks output length
- Splits into chunks if needed
- Creates file attachments for large output
- Adds status indicators (✅/❌)

## Configuration Flow

```
.env file
    │
    │ DISCORD_TOKEN=...
    │
    ▼
Environment Variables
    │
    │ std::env::var("DISCORD_TOKEN")
    │
    ▼
main.rs
    │
    │ match "discord-bot" => ...
    │
    ▼
discord::start_discord_bot(token)
    │
    │ Create ClientBuilder with token
    │
    ▼
Serenity Client
    │
    │ Authenticate with Discord
    │
    ▼
Bot Online
```

## Error Handling Flow

```
Command Execution
    │
    ├─▶ Subprocess fails
    │   │
    │   ├─▶ Capture exit code
    │   │
    │   ├─▶ Capture stderr
    │   │
    │   └─▶ Format error message
    │       │
    │       └─▶ Send to Discord with ❌
    │
    ├─▶ Output too large
    │   │
    │   └─▶ Send as file attachment
    │
    ├─▶ Discord API error
    │   │
    │   ├─▶ Log error
    │   │
    │   └─▶ Return error to framework
    │
    └─▶ Subprocess spawn fails
        │
        ├─▶ Log error
        │
        └─▶ Send error message to Discord
```

## Concurrency Model

```
Bot Process (Single Instance)
    │
    ├─▶ Tokio Runtime
    │   │
    │   ├─▶ Serenity Event Loop (1 task)
    │   │   │
    │   │   └─▶ Handles all Discord events
    │   │
    │   ├─▶ Command Handler 1 (spawned task)
    │   │   │
    │   │   └─▶ User A's /test-swap
    │   │
    │   ├─▶ Command Handler 2 (spawned task)
    │   │   │
    │   │   └─▶ User B's /test-swap-all
    │   │
    │   └─▶ Command Handler N (spawned task)
    │       │
    │       └─▶ User N's /ping
    │
    └─▶ Multiple Cargo Subprocesses
        │
        ├─▶ Subprocess 1 (User A's test)
        │
        ├─▶ Subprocess 2 (User B's test)
        │
        └─▶ Subprocess N (User N's test)
```

Each command handler runs in its own async task, allowing multiple users to execute commands simultaneously without blocking each other.

## Security Boundaries

```
Discord (Untrusted)
    │
    │ Slash Commands Only
    │ (No arbitrary text)
    │
    ▼
Bot (Trusted)
    │
    │ Validates command structure
    │ (Poise framework)
    │
    ▼
Command Handler (Trusted)
    │
    │ Hardcoded command: "cargo"
    │ Hardcoded args: ["run", "--release", "--", "test-swap", ...]
    │ Working dir: Project root only
    │
    ▼
Subprocess (Trusted)
    │
    │ Executes cargo (trusted binary)
    │ In project directory only
    │
    ▼
CLI Application (Trusted)
    │
    │ Your existing code
    │
    ▼
External APIs (Varies)
    │
    └─▶ Garden Finance API
```

No arbitrary command execution - only predefined cargo commands with validated parameters.

## Performance Characteristics

### Latency
- Command reception: < 100ms (Discord → Bot)
- Command parsing: < 10ms (Poise)
- Subprocess spawn: < 100ms (tokio)
- Swap execution: 30s - 15min (depends on blockchain)
- Response send: < 100ms (Bot → Discord)

### Throughput
- Concurrent commands: Limited by system resources
- Typical: 10-100 concurrent swaps
- Bottleneck: Blockchain confirmation times, not bot

### Resource Usage
- Memory: ~50MB (bot) + ~100MB per subprocess
- CPU: Minimal (mostly I/O bound)
- Network: WebSocket (persistent) + HTTP (API calls)

## Deployment Architecture

```
Production Server
    │
    ├─▶ systemd service (or equivalent)
    │   │
    │   └─▶ cargo run -- discord-bot
    │       │
    │       ├─▶ Reads .env
    │       ├─▶ Connects to Discord
    │       └─▶ Listens for commands
    │
    ├─▶ Log files
    │   │
    │   └─▶ /var/log/discord-bot/
    │
    └─▶ Database
        │
        └─▶ garden_swaps.db
```

## Monitoring Points

1. **Bot Health**: /ping command response time
2. **Command Success Rate**: Exit codes from subprocesses
3. **Response Times**: Time from command to response
4. **Error Rates**: Failed commands / total commands
5. **Concurrent Users**: Active command handlers
6. **Resource Usage**: Memory, CPU, disk I/O

---

This architecture provides a clean separation between the Discord bot (new code) and your existing CLI application (unchanged), with proper async execution and error handling throughout.
