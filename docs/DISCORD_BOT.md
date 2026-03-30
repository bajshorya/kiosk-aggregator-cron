# Discord Bot Integration

This project includes a Discord bot that allows you to execute swap tests directly from Discord using slash commands.

## Features

- Execute single swap tests via `/test-swap`
- Run all swap tests in batch mode via `/test-swap-all`
- Health check via `/ping`
- Automatic output handling (splits long messages or sends as file attachment)
- Non-blocking async execution (bot stays responsive during long-running tests)

## Setup

### 1. Create a Discord Bot

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Click "New Application" and give it a name
3. Go to the "Bot" section and click "Add Bot"
4. Under "Token", click "Reset Token" and copy the token
5. Save the token in your `.env` file as `DISCORD_TOKEN`

### 2. Configure Bot Permissions

In the Discord Developer Portal:

1. Go to "OAuth2" → "URL Generator"
2. Select scopes:
   - `bot`
   - `applications.commands`
3. Select bot permissions:
   - Send Messages
   - Attach Files
   - Use Slash Commands
4. Copy the generated URL and open it in your browser to invite the bot to your server

### 3. Add Token to Environment

Add this line to your `.env` file:

```env
DISCORD_TOKEN=your_bot_token_here
```

### 4. Start the Bot

```bash
cargo run -- discord-bot
```

The bot will:
- Connect to Discord
- Register slash commands globally (may take up to 1 hour to propagate)
- Start listening for commands

## Available Commands

### `/ping`

Health check command. The bot will reply with "pong 🏓".

**Usage:**
```
/ping
```

### `/test-swap`

Execute a single swap test between two chains/tokens.

**Parameters:**
- `from_asset` (required): Source chain and token (e.g., `ethereum_sepolia:eth`)
- `to_asset` (required): Destination chain and token (e.g., `base_sepolia:wbtc`)
- `amount` (optional): Custom amount in smallest units

**Examples:**
```
/test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc
/test-swap from_asset:solana_testnet:sol to_asset:arbitrum_sepolia:usdc
/test-swap from_asset:ethereum_sepolia:usdc to_asset:solana_testnet:usdc amount:200000000
```

**What it does:**
Executes: `cargo run --release -- test-swap <from_asset> <to_asset> [amount]`

### `/test-swap-all`

Run all swap tests in batch mode (run-once).

**Usage:**
```
/test-swap-all
```

**What it does:**
Executes: `cargo run -- run-once`

## Output Handling

The bot intelligently handles command output:

1. **Short output (< 2000 chars)**: Sent as a single Discord message
2. **Medium output (2000-10000 chars)**: Split into multiple messages
3. **Long output (> 10000 chars)**: Sent as a `.txt` file attachment

All commands use `ctx.defer()` to prevent Discord timeouts during long-running operations.

## Technical Details

### Architecture

```
Discord                        Your Machine
  │                                │
  │  /test-swap from:eth to:wbtc  │
  │ ─────────────────────────────▶│
  │                                │  tokio::process::Command
  │                                │  cargo run --release -- test-swap ethereum_sepolia:eth base_sepolia:wbtc
  │                                │  (captures stdout + stderr)
  │  ◀─────────────────────────────│
  │  "✅ Output: [full CLI output]"│
```

### Dependencies

- `poise` (0.6): Slash command framework
- `serenity` (0.12): Discord API client
- `tokio`: Async runtime for non-blocking subprocess execution

### File Structure

```
src/discord/
├── mod.rs                      # Module exports
├── bot.rs                      # Bot initialization and framework setup
└── commands/
    ├── mod.rs                  # Command exports
    └── test_swap.rs            # Slash command implementations
```

### How It Works

1. Bot connects to Discord using the token from `DISCORD_TOKEN`
2. Slash commands are registered globally on startup
3. When a command is received:
   - `ctx.defer()` is called to prevent timeout
   - `tokio::process::Command` spawns the CLI subprocess
   - Working directory is set to project root (where `Cargo.toml` is)
   - stdout and stderr are captured asynchronously
   - Output is sent back to Discord (split or as file if needed)

### Non-Blocking Execution

The bot uses `tokio::process::Command` instead of `std::process::Command` to ensure:
- The bot stays responsive during long-running tests
- Multiple users can execute commands simultaneously
- Discord doesn't timeout waiting for responses

## Troubleshooting

### Bot doesn't respond to commands

1. Check that the bot is online in your Discord server
2. Verify `DISCORD_TOKEN` is set correctly in `.env`
3. Wait up to 1 hour for slash commands to propagate globally
4. Check bot logs for errors

### "DISCORD_TOKEN environment variable not set"

Add the token to your `.env` file:
```env
DISCORD_TOKEN=your_bot_token_here
```

### Commands fail with "cargo: command not found"

Ensure `cargo` is in your system PATH. The bot executes cargo commands in a subprocess.

### Output is truncated

If output exceeds 10,000 characters, it's automatically sent as a `.txt` file attachment.

## Security Notes

- Never commit your Discord token to git
- The bot only has access to commands you explicitly define
- All subprocess execution happens in the project root directory
- The bot uses non-privileged Discord intents (no access to message content)

## Development

To add new commands:

1. Add the command function in `src/discord/commands/test_swap.rs` (or create a new file)
2. Export it in `src/discord/commands/mod.rs`
3. Register it in `src/discord/bot.rs` in the `commands` vector

Example:
```rust
#[poise::command(slash_command)]
pub async fn my_command(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Hello!").await?;
    Ok(())
}
```
