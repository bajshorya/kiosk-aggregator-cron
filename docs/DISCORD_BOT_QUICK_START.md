# Discord Bot Quick Start

Get your Discord bot running in 5 minutes!

## Step 1: Create Discord Bot (2 minutes)

1. Go to https://discord.com/developers/applications
2. Click "New Application" → Name it (e.g., "Garden Swap Tester")
3. Go to "Bot" tab → Click "Add Bot"
4. Click "Reset Token" → Copy the token
5. Go to "OAuth2" → "URL Generator":
   - Check: `bot` and `applications.commands`
   - Check: "Send Messages", "Attach Files", "Use Slash Commands"
   - Copy the generated URL and open it to invite bot to your server

## Step 2: Configure Environment (30 seconds)

Add to your `.env` file:

```env
DISCORD_TOKEN=paste_your_token_here
```

## Step 3: Start the Bot (30 seconds)

```bash
cargo run -- discord-bot
```

You should see:
```
Starting Discord bot...
Bot is ready! Registering slash commands globally...
Slash commands registered successfully
```

## Step 4: Use Commands in Discord

In your Discord server, type `/` and you'll see:

- `/ping` - Test if bot is alive
- `/test-swap` - Run a single swap test
- `/test-swap-all` - Run all swap tests

### Example Commands

```
/ping
```

```
/test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc
```

```
/test-swap from_asset:solana_testnet:sol to_asset:arbitrum_sepolia:usdc amount:1000000000
```

```
/test-swap-all
```

## Notes

- Slash commands may take up to 1 hour to appear globally (Discord limitation)
- The bot will defer responses to prevent timeouts during long operations
- Large outputs are automatically sent as `.txt` file attachments
- The bot stays responsive and can handle multiple commands simultaneously

## Troubleshooting

**Commands don't appear?**
- Wait up to 1 hour for Discord to propagate slash commands
- Restart Discord client
- Check bot has proper permissions in your server

**Bot doesn't respond?**
- Check bot is online (green dot in member list)
- Verify `DISCORD_TOKEN` in `.env` is correct
- Check bot logs for errors

**"cargo: command not found"?**
- Ensure cargo is in your system PATH
- Bot needs to run from the project root directory

## What Happens Behind the Scenes

```
You type: /test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc

Bot executes: cargo run --release -- test-swap ethereum_sepolia:eth base_sepolia:wbtc

Bot captures: All stdout and stderr output

Bot replies: Full output in Discord (split or as file if too long)
```

That's it! Your Discord bot is now ready to execute swap tests remotely. 🚀
