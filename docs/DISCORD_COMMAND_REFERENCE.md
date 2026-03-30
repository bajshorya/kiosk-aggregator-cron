# Discord Bot Command Reference

Quick reference for all Discord slash commands and their CLI equivalents.

## Command Mappings

### `/ping`

**Description**: Health check to verify bot is online and responsive.

**Discord Command**:
```
/ping
```

**CLI Equivalent**: N/A (Discord-only command)

**Response**: `pong 🏓`

**Use Case**: Verify bot is running and can respond to commands.

---

### `/test-swap`

**Description**: Execute a single swap test between two chains/tokens.

**Discord Command**:
```
/test-swap from_asset:<chain:token> to_asset:<chain:token> [amount:<value>]
```

**Parameters**:
- `from_asset` (required): Source chain and token in format `chain:token`
- `to_asset` (required): Destination chain and token in format `chain:token`
- `amount` (optional): Custom amount in smallest units (e.g., satoshis, wei, lamports)

**CLI Equivalent**:
```bash
cargo run --release -- test-swap <from_asset> <to_asset> [amount]
```

**Examples**:

1. **Basic swap (ETH to WBTC)**:
   ```
   /test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc
   ```
   CLI: `cargo run --release -- test-swap ethereum_sepolia:eth base_sepolia:wbtc`

2. **Solana to Arbitrum**:
   ```
   /test-swap from_asset:solana_testnet:sol to_asset:arbitrum_sepolia:usdc
   ```
   CLI: `cargo run --release -- test-swap solana_testnet:sol arbitrum_sepolia:usdc`

3. **With custom amount**:
   ```
   /test-swap from_asset:ethereum_sepolia:usdc to_asset:solana_testnet:usdc amount:200000000
   ```
   CLI: `cargo run --release -- test-swap ethereum_sepolia:usdc solana_testnet:usdc 200000000`

4. **Bitcoin to Base**:
   ```
   /test-swap from_asset:bitcoin_testnet:btc to_asset:base_sepolia:wbtc
   ```
   CLI: `cargo run --release -- test-swap bitcoin_testnet:btc base_sepolia:wbtc`

**Response**: Full CLI output including:
- Swap pair
- Status (Completed, Failed, TimedOut)
- Order ID
- Duration
- Transaction hashes
- Error messages (if any)

**Use Case**: Test a specific swap pair to verify it works correctly.

---

### `/test-swap-all`

**Description**: Run all configured swap tests in batch mode (run-once).

**Discord Command**:
```
/test-swap-all
```

**Parameters**: None

**CLI Equivalent**:
```bash
cargo run -- run-once
```

**Response**: Complete batch test results including:
- Run ID
- Total swaps
- Completed count
- Failed count
- Timed out count
- Detailed results for each swap pair

**Example Output**:
```
═══ Final Run Summary ═══
Run ID   : 4fb3fd66-718b-4975-8657-245c8e665455
Total    : 16
Completed: 12
Failed   : 2
Timed Out: 2
Pending  : 0

✅ ethereum_sepolia:wbtc -> base_sepolia:wbtc          | Completed    |
✅ base_sepolia:wbtc -> arbitrum_sepolia:wbtc          | Completed    |
❌ base_sepolia:wbtc -> ethereum_sepolia:wbtc          | Failed       | Insufficient balance
⏰ arbitrum_sepolia:wbtc -> base_sepolia:wbtc          | TimedOut     |
...
```

**Use Case**: Run comprehensive tests of all swap pairs to verify system health.

---

## Supported Chain:Token Pairs

### Bitcoin
- `bitcoin_testnet:btc`

### Ethereum Sepolia
- `ethereum_sepolia:eth`
- `ethereum_sepolia:wbtc`
- `ethereum_sepolia:usdc`

### Base Sepolia
- `base_sepolia:wbtc`
- `base_sepolia:usdc`

### Arbitrum Sepolia
- `arbitrum_sepolia:wbtc`
- `arbitrum_sepolia:usdc`

### Solana Testnet
- `solana_testnet:sol`
- `solana_testnet:usdc`

### Starknet Sepolia
- `starknet_sepolia:wbtc`

### Tron Shasta
- `tron_shasta:usdt`
- `tron_shasta:wbtc`

### Other Testnets
- `alpen_testnet:sbtc`
- `alpen_testnet:usdc`
- `bnb_testnet:wbtc`
- `citrea_testnet:usdc`
- `monad_testnet:usdc`
- `xrpl_testnet:xrp`

---

## Output Handling

The bot intelligently handles output based on size:

### Short Output (< 2000 characters)
Sent as a single Discord message.

### Medium Output (2000-10000 characters)
Split into multiple messages (up to 5 messages).

### Long Output (> 10000 characters)
Sent as a `.txt` file attachment.

---

## Response Format

All command outputs include:

1. **STDOUT Section**:
   ```
   **STDOUT:**
   ```
   [Command output]
   ```
   ```

2. **STDERR Section** (if any):
   ```
   **STDERR:**
   ```
   [Error output]
   ```
   ```

3. **Status**:
   - ✅ Command completed successfully (exit code: 0)
   - ❌ Command failed (exit code: 1)

---

## Timing and Timeouts

### Discord Timeout Prevention
- All commands use `ctx.defer()` to prevent Discord's 3-second timeout
- Deferred responses have 15 minutes to complete
- Long-running swaps (up to 15 minutes) are fully supported

### Swap Timeouts
- Default swap timeout: 900 seconds (15 minutes)
- Configurable via `SWAP_TIMEOUT_SECS` in `.env`
- Status polling interval: 15 seconds (configurable via `POLL_INTERVAL_SECS`)

---

## Error Handling

### Common Errors

1. **"DISCORD_TOKEN environment variable not set"**
   - Solution: Add `DISCORD_TOKEN=your_token` to `.env`

2. **"cargo: command not found"**
   - Solution: Ensure cargo is in system PATH

3. **"Insufficient balance"**
   - Solution: Add testnet tokens to your wallet

4. **"Swap timed out"**
   - Solution: Increase `SWAP_TIMEOUT_SECS` or check network status

### Error Output
All errors are captured and displayed in the Discord response:
```
❌ Command failed (exit code: 1)

**STDERR:**
```
Error: Insufficient balance for swap
```
```

---

## Best Practices

### 1. Test with /ping First
Always verify bot is responsive before running swap tests:
```
/ping
```

### 2. Start with Single Swaps
Test individual pairs before running batch tests:
```
/test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc
```

### 3. Monitor Output
Watch for:
- ✅ Completed swaps
- ❌ Failed swaps (check error messages)
- ⏰ Timed out swaps (may need longer timeout)

### 4. Use Custom Amounts Carefully
Default amounts are optimized for testnet. Only use custom amounts if needed:
```
/test-swap from_asset:ethereum_sepolia:usdc to_asset:solana_testnet:usdc amount:200000000
```

### 5. Batch Tests for CI/CD
Use `/test-swap-all` for comprehensive testing:
```
/test-swap-all
```

---

## Advanced Usage

### Parallel Execution
The bot can handle multiple commands simultaneously:
- User A: `/test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc`
- User B: `/test-swap from_asset:solana_testnet:sol to_asset:arbitrum_sepolia:usdc`
- Both execute in parallel without blocking

### Command History
All swap results are stored in the SQLite database:
```bash
# View history via CLI
cargo run -- history
```

### Integration with CI/CD
Use Discord webhooks to trigger tests from CI/CD pipelines:
```bash
# Example: Trigger test after deployment
curl -X POST "https://discord.com/api/webhooks/..." \
  -H "Content-Type: application/json" \
  -d '{"content": "/test-swap-all"}'
```

---

## Troubleshooting

### Commands Don't Appear
- Wait up to 1 hour for global command registration
- Restart Discord client
- Check bot has proper permissions

### Bot Doesn't Respond
- Verify bot is online (green dot in member list)
- Check `DISCORD_TOKEN` is correct
- Review bot logs for errors

### Output Truncated
- Large outputs are automatically sent as file attachments
- Download the `.txt` file for complete output

### Swap Fails
- Check wallet has sufficient testnet tokens
- Verify RPC endpoints are accessible
- Review error message in output

---

## Quick Reference Table

| Discord Command | CLI Equivalent | Purpose |
|----------------|----------------|---------|
| `/ping` | N/A | Health check |
| `/test-swap from:A to:B` | `cargo run --release -- test-swap A B` | Single swap test |
| `/test-swap from:A to:B amount:X` | `cargo run --release -- test-swap A B X` | Single swap with custom amount |
| `/test-swap-all` | `cargo run -- run-once` | All swaps batch test |

---

## Support

For more information:
- **Setup Guide**: `docs/DISCORD_BOT_QUICK_START.md`
- **Full Documentation**: `docs/DISCORD_BOT.md`
- **Implementation Details**: `docs/DISCORD_BOT_IMPLEMENTATION.md`
- **Main README**: `README.md`
