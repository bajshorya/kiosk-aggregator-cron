# Discord Bot Testing Guide

## Overview

This guide helps you test and troubleshoot the Discord bot commands.

## Available Commands

### 1. `/ping`
**Purpose**: Health check to verify bot is online

**Usage:**
```
/ping
```

**Expected Response:**
```
pong 🏓
```

**Troubleshooting:**
- If no response: Bot is offline or not connected
- Check bot process is running
- Verify DISCORD_TOKEN is correct

---

### 2. `/test-swap`
**Purpose**: Test a single swap between two assets

**Usage:**
```
/test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc
```

**With custom amount:**
```
/test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc amount:1000000
```

**Expected Response:**
- Discord embed with swap details
- Status indicator (✅ Completed, ❌ Failed, ⏰ TimedOut)
- Order ID, duration, transaction hashes
- Color-coded based on status (green=success, red=failed, orange=timeout)

**What it executes:**
```bash
cargo run --release -- test-swap ethereum_sepolia:eth base_sepolia:wbtc
```

**Troubleshooting:**

1. **Command takes too long (> 15 minutes)**
   - Discord will show "Thinking..." then timeout
   - The swap may still be running in the background
   - Check the bot logs for progress

2. **No output shown**
   - Check if compilation messages are being filtered
   - Verify the CLI command works: `cargo run -- test-swap ethereum_sepolia:eth base_sepolia:wbtc`
   - Check bot logs for errors

3. **"Command failed" message**
   - Check the error details in the output
   - Common issues:
     - Insufficient balance
     - Invalid asset pair
     - Network connectivity issues
     - RPC endpoint problems

---

### 3. `/test-swap-all`
**Purpose**: Run all configured swap tests in batch mode

**Usage:**
```
/test-swap-all
```

**Expected Response:**
- Discord embed with batch summary
- Total swaps, completed, failed, timed out counts
- Color-coded status
- List of all swap results

**What it executes:**
```bash
cargo run --release -- run-once
```

**Troubleshooting:**

1. **Command times out**
   - Batch tests can take 30+ minutes
   - Discord has a 15-minute interaction timeout
   - Solution: The bot defers the response, giving 15 minutes
   - If still timing out, consider reducing the number of swaps

2. **No output or partial output**
   - Check if the `run-once` command works manually:
     ```bash
     cargo run -- run-once
     ```
   - Verify output is being captured correctly
   - Check bot logs for errors

3. **Output too large**
   - If output exceeds Discord's limits, it will be sent as a file attachment
   - Look for "📄 Output is too large, attached as file:"

4. **Some swaps fail**
   - This is normal for testnet
   - Check individual swap errors in the output
   - Common causes:
     - Insufficient testnet tokens
     - Network congestion
     - RPC rate limits

---

## Testing Checklist

### Before Testing

- [ ] Bot is running (`cargo run -- discord-bot`)
- [ ] Bot shows "Connected" in Discord server member list
- [ ] Commands are registered (may take up to 1 hour for global commands)
- [ ] DISCORD_TOKEN is set correctly in .env
- [ ] All wallet private keys are configured
- [ ] RPC endpoints are accessible

### Test Sequence

1. **Test bot connectivity:**
   ```
   /ping
   ```
   Expected: "pong 🏓" response

2. **Test single swap (fast pair):**
   ```
   /test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc
   ```
   Expected: Embed with swap result in 2-5 minutes

3. **Test autocomplete:**
   - Type `/test-swap` and click on `from_asset`
   - Verify dropdown appears with asset list
   - Type "eth" and verify filtering works
   - Select an asset

4. **Test batch swaps (optional, takes long time):**
   ```
   /test-swap-all
   ```
   Expected: Batch summary embed in 15-30 minutes

### After Testing

- [ ] Check bot logs for any errors
- [ ] Verify database has new records
- [ ] Check wallet balances if needed

---

## Manual Testing

### Test CLI Commands Directly

1. **Test single swap:**
   ```bash
   cargo run -- test-swap ethereum_sepolia:eth base_sepolia:wbtc
   ```

2. **Test batch swaps:**
   ```bash
   cargo run -- run-once
   ```

3. **Check output format:**
   - Look for "═══ Swap Test Result ═══" or "═══ Final Run Summary ═══"
   - Verify all fields are present (Pair, Status, Order ID, etc.)

### Test Output Parsing

The bot parses CLI output to create embeds. Test that parsing works:

1. Run a swap manually and save output:
   ```bash
   cargo run -- test-swap ethereum_sepolia:eth base_sepolia:wbtc > test_output.txt
   ```

2. Check if output contains expected markers:
   - "═══" for section headers
   - "Pair:", "Status:", "Order ID:", etc.
   - Status values: "Completed", "Failed", "TimedOut"

---

## Common Issues

### Issue: Commands don't appear in Discord

**Symptoms:**
- Typing `/` doesn't show bot commands
- Commands show as "Unknown command"

**Solutions:**
1. Wait up to 1 hour for global command registration
2. Restart Discord client
3. Check bot has "Use Application Commands" permission
4. Restart bot to force re-registration

---

### Issue: Bot responds but output is empty

**Symptoms:**
- Bot says "Command completed successfully"
- But no actual swap details shown

**Causes:**
- Output filtering is too aggressive
- CLI command produces no output
- Parsing fails to extract information

**Solutions:**
1. Check bot logs for the actual output
2. Run CLI command manually to verify output
3. Check `filter_compilation_output()` function
4. Verify `parse_swap_output_to_embed()` function

---

### Issue: Compilation messages in output

**Symptoms:**
- Discord shows "Compiling...", "Building...", etc.
- Output is cluttered with cargo messages

**Causes:**
- Filter function not working
- Compilation happening during command execution

**Solutions:**
1. Ensure release binary is pre-built: `cargo build --release`
2. Check `filter_compilation_output()` function
3. Verify stderr is being filtered before sending to Discord

---

### Issue: Swap takes too long

**Symptoms:**
- Discord shows "Thinking..." for > 5 minutes
- Eventually times out

**Causes:**
- Blockchain confirmation delays
- Network congestion
- Testnet issues

**Solutions:**
1. Increase `SWAP_TIMEOUT_SECS` in .env
2. Use faster chains (Ethereum Sepolia, Base Sepolia)
3. Check RPC endpoints are responsive
4. Consider using mainnet (faster but costs real money)

---

## Debugging

### Enable Debug Logging

1. Set environment variable:
   ```bash
   $env:RUST_LOG="debug"
   cargo run -- discord-bot
   ```

2. Check logs for:
   - Command execution
   - Output capture
   - Parsing results
   - Discord API calls

### Check Bot Process

```bash
# List running processes
Get-Process | Where-Object {$_.ProcessName -like "*cargo*"}

# Check bot logs
# (if running as background process, use getProcessOutput)
```

### Verify Environment

```bash
# Check DISCORD_TOKEN is set
echo $env:DISCORD_TOKEN

# Check .env file
Get-Content .env | Select-String "DISCORD_TOKEN"

# Test cargo commands
cargo run -- test-swap ethereum_sepolia:eth base_sepolia:wbtc
```

---

## Performance Tips

### Speed Up Commands

1. **Pre-build release binary:**
   ```bash
   cargo build --release
   ```
   This avoids compilation during command execution

2. **Use faster chains:**
   - Ethereum Sepolia (fast)
   - Base Sepolia (fast)
   - Arbitrum Sepolia (fast)
   - Avoid: Bitcoin (slow), Solana (can be slow)

3. **Reduce swap timeout:**
   ```env
   SWAP_TIMEOUT_SECS=300  # 5 minutes instead of 15
   ```

### Reduce Output Size

1. **Filter unnecessary output:**
   - Compilation messages (already filtered)
   - Debug logs (use INFO level)
   - Verbose transaction details

2. **Use embeds instead of text:**
   - Embeds are more compact
   - Better visual presentation
   - Easier to parse

---

## Expected Timings

| Command | Expected Duration | Notes |
|---------|------------------|-------|
| `/ping` | < 1 second | Instant response |
| `/test-swap` (EVM) | 2-5 minutes | Fast chains |
| `/test-swap` (Bitcoin) | 10-30 minutes | Slow confirmations |
| `/test-swap-all` | 15-45 minutes | Depends on number of swaps |

---

## Success Criteria

A successful test should show:

1. **For `/ping`:**
   - ✅ Immediate "pong 🏓" response

2. **For `/test-swap`:**
   - ✅ Discord embed with colored border
   - ✅ Status emoji (✅/❌/⏰)
   - ✅ Swap pair displayed
   - ✅ Order ID (if available)
   - ✅ Duration shown
   - ✅ Transaction hashes (if completed)
   - ✅ No compilation messages

3. **For `/test-swap-all`:**
   - ✅ Batch summary embed
   - ✅ Total/Completed/Failed/TimedOut counts
   - ✅ Color-coded status
   - ✅ Footer with summary message
   - ✅ No compilation messages

---

## Next Steps

After successful testing:

1. **Monitor production usage:**
   - Check bot logs regularly
   - Monitor swap success rates
   - Track response times

2. **Optimize as needed:**
   - Adjust timeouts
   - Add more asset pairs
   - Improve error messages

3. **Add features:**
   - Progress updates during long swaps
   - Swap history command
   - Balance checking command
   - Statistics dashboard

---

**Last Updated:** 2026-03-30
**Version:** 1.3.0
