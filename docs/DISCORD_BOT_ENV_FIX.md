# Discord Bot Environment Variable Fix

## Issue

The DISCORD_TOKEN environment variable from the `.env` file was not being loaded properly by the `dotenv` crate, even though other variables (like SOLANA_PRIVATE_KEY) were loading correctly.

## Root Cause

The issue appears to be related to how the `dotenv` crate handles environment variables in certain scenarios. While `dotenv::dotenv()` is called at the start of `main()` and in `AppConfig::from_env()`, the DISCORD_TOKEN variable specifically wasn't being picked up.

## Solution

### Option 1: PowerShell Start Script (Recommended)

Use the provided `start-discord-bot.ps1` script which manually loads all environment variables from the `.env` file:

```powershell
.\start-discord-bot.ps1
```

The script:
1. Reads the `.env` file line by line
2. Parses each `KEY=VALUE` pair
3. Sets them as process environment variables
4. Starts the bot with `cargo run --release -- discord-bot`

### Option 2: Manual Environment Variable

Set the DISCORD_TOKEN manually before running:

```powershell
$env:DISCORD_TOKEN='YOUR_TOKEN_HERE'
cargo run --release -- discord-bot
```

### Option 3: Direct Binary Execution

```powershell
$env:DISCORD_TOKEN='YOUR_TOKEN_HERE'
.\target\release\kiosk-aggregator-cron.exe discord-bot
```

## Code Changes Made

1. **Added early dotenv loading in main.rs**:
   ```rust
   #[tokio::main]
   async fn main() -> Result<()> {
       // Load .env file first, before anything else
       dotenv::dotenv().ok();
       // ... rest of initialization
   }
   ```

2. **Added debug logging** to help diagnose the issue:
   ```rust
   match std::env::var("DISCORD_TOKEN") {
       Ok(ref token) => {
           info!("DISCORD_TOKEN found in environment (length: {})", token.len());
       }
       Err(_) => {
           error!("DISCORD_TOKEN not found in environment");
       }
   }
   ```

3. **Created PowerShell start script** (`start-discord-bot.ps1`) that reliably loads all environment variables

## Verification

When the bot starts successfully, you should see:

```
2026-03-31T05:43:04.949138Z  INFO DISCORD_TOKEN found in environment (length: 72)
Starting Discord bot...
Press Ctrl+C to stop.
2026-03-31T05:43:04.949453Z  INFO Starting Discord bot...
2026-03-31T05:43:05.747783Z  INFO Discord client created, starting event loop...
2026-03-31T05:43:07.804811Z  INFO Bot is ready! Registering slash commands globally...
2026-03-31T05:43:08.719461Z  INFO Slash commands registered successfully
```

## Files Modified

- `src/main.rs` - Added early dotenv loading and debug logging
- `.env` - Added comment header for DISCORD_TOKEN
- `start-discord-bot.ps1` - Created PowerShell start script

## Related Issues

This issue may be related to:
- Windows-specific dotenv behavior
- PowerShell environment variable handling
- Cargo subprocess environment inheritance

## Workaround Status

✅ **Working**: Using `start-discord-bot.ps1` script
✅ **Working**: Manual environment variable setting
⚠️ **Partial**: Direct `cargo run` may not work without manual env var

## Future Improvements

Consider:
1. Investigating dotenv crate alternatives (e.g., `dotenvy`)
2. Adding environment variable validation at startup
3. Creating platform-specific start scripts (`.bat` for Windows, `.sh` for Linux/Mac)
4. Adding a configuration file loader as fallback
