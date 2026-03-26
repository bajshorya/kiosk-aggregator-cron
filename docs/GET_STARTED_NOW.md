# 🚀 Get Started Now - Quick Action Guide

## ⚡ Immediate Actions (5 minutes)

### Step 1: Get Testnet ETH
Your wallet needs testnet ETH for gas fees.

**Wallet Address**: `0x55D246A6c46C4371ffcb6e18cD1D10Eca2d6F406`

**Faucets** (try all for maximum ETH):
1. https://sepoliafaucet.com/
2. https://www.alchemy.com/faucets/ethereum-sepolia
3. https://faucets.chain.link/sepolia
4. https://sepolia-faucet.pk910.de/

**Expected**: 0.1-0.5 ETH per faucet (enough for 20-100 swaps)

### Step 2: Test Your First Swap
Once you have testnet ETH (check balance on https://sepolia.etherscan.io/address/0x55D246A6c46C4371ffcb6e18cD1D10Eca2d6F406):

```bash
# Test ETH to Solana swap
cargo run --release -- test-swap ethereum_sepolia:eth solana_testnet:sol
```

**Expected Output**:
```
✅ Quote received
✅ Order created
✅ Transaction sent
✅ Swap completed
```

### Step 3: Test Bitcoin Manual Deposit
While waiting for ETH faucets, test Bitcoin:

```bash
# Start Bitcoin swap
cargo run --release -- test-swap bitcoin_testnet:btc base_sepolia:wbtc
```

**You'll get a deposit address like**:
```
Send 50000 sats to: tb1p324m5xa7zxg7f4axyh8x7nnwyyrw0m4nw4gx7w7ujkkpea70d46qstxwee0
```

**Get testnet Bitcoin**:
- https://testnet-faucet.com/btc-testnet/
- https://coinfaucet.eu/en/btc-testnet/

**Send to the address**, then watch the swap complete automatically!

## 📊 Check Your Progress

### View Swap History
```bash
cargo run --release -- history
```

### List All Available Pairs
```bash
cargo run --release -- list-swaps
# Shows all 24 swap pairs
```

### Check Database
```bash
sqlite3 garden_swaps.db "SELECT * FROM swap_results ORDER BY started_at DESC LIMIT 10;"
```

## 🎯 Success Criteria

### ✅ You're successful when:
1. ETH → Solana swap completes end-to-end
2. Bitcoin → Base swap completes with manual deposit
3. Database shows completed swaps
4. No errors in logs

### ⚠️ If you see errors:
- **"Insufficient gas"**: Get more testnet ETH from faucets
- **"Insufficient liquidity"**: Try different pair or wait for liquidity
- **"Gasless not available"**: Expected - using gas-based swaps
- **"Failed to send transaction"**: Check RPC URL or try again

## 🔄 Run All Swaps

Once you have testnet ETH and Bitcoin:

```bash
# Run all 24 swap pairs
cargo run --release -- run-once
```

**This will**:
- Test all swap pairs concurrently
- Show progress for each swap
- Save results to database
- Print final summary

**Expected time**: 15-30 minutes (depending on network)

## 📈 Monitor Progress

### Real-time Logs
```bash
# Run with debug logging
RUST_LOG=debug cargo run --release -- test-swap ethereum_sepolia:eth solana_testnet:sol
```

### Check Transactions
- **Ethereum Sepolia**: https://sepolia.etherscan.io/address/0x55D246A6c46C4371ffcb6e18cD1D10Eca2d6F406
- **Solana Testnet**: https://explorer.solana.com/address/5ymiDmUNjH1ehfVaR9W95kNXyKuaLsVPGBS7dXLP41ny?cluster=testnet
- **Bitcoin Testnet**: https://blockstream.info/testnet/address/tb1qypyl87pagnmcted8d8gnmxxps7dyqstllhhe9z

## 🎉 What's Working Right Now

### ✅ Fully Functional
- Quote API (all pairs)
- Order creation (all pairs)
- Manual deposit (Bitcoin, Litecoin)
- Gas-based swaps (EVM, Solana)
- Database tracking
- History viewing
- Scheduler

### 🚧 Needs Testnet Tokens
- ETH → Solana (needs ETH)
- EVM ↔ EVM (needs ETH)
- Solana → ETH (needs SOL)

### 🔧 Needs Implementation
- Starknet (stub created)
- Tron (stub created)
- Sui (stub created)

## 💡 Pro Tips

### Get More Testnet Tokens
```bash
# Solana
solana airdrop 2 5ymiDmUNjH1ehfVaR9W95kNXyKuaLsVPGBS7dXLP41ny --url testnet

# Check balances
# ETH: https://sepolia.etherscan.io/address/0x55D246A6c46C4371ffcb6e18cD1D10Eca2d6F406
# SOL: solana balance 5ymiDmUNjH1ehfVaR9W95kNXyKuaLsVPGBS7dXLP41ny --url testnet
```

### Test Specific Pairs
```bash
# EVM to EVM (fast, cheap)
cargo run --release -- test-swap ethereum_sepolia:wbtc base_sepolia:wbtc

# Solana to Bitcoin (cross-chain)
cargo run --release -- test-swap solana_testnet:sol bitcoin_testnet:btc

# Bitcoin to EVM (manual deposit)
cargo run --release -- test-swap bitcoin_testnet:btc ethereum_sepolia:wbtc
```

### Optimize for Speed
```bash
# Build in release mode (10x faster)
cargo build --release

# Run with minimal logging
RUST_LOG=info cargo run --release -- run-once
```

## 🐛 Troubleshooting

### Problem: "Failed to send transaction"
**Solution**: Get testnet ETH from faucets (see Step 1)

### Problem: "Insufficient liquidity"
**Solution**: Try different pair or wait. Some pairs have low liquidity on testnet.

### Problem: "Order creation failed"
**Solution**: Check API is accessible. Try again in a few seconds.

### Problem: "Gasless not available"
**Solution**: This is expected. System automatically falls back to gas-based swaps.

### Problem: Compilation errors
**Solution**: 
```bash
cargo clean
cargo build --release
```

## 📚 Next Steps After Success

### 1. Test All Pairs
```bash
cargo run --release -- run-once
```

### 2. Set Up Scheduler
```bash
# Edit .env to set cron schedule
SCHEDULER_CRON=0 0 */5 * * *

# Run scheduler
cargo run --release
```

### 3. Add More Chains
See `docs/ADDING_CHAIN_SUPPORT.md` for:
- Starknet implementation
- Tron implementation
- Sui implementation

### 4. Production Deployment
- Switch to mainnet endpoints
- Use real funds
- Set up monitoring
- Implement proper security

## 🎯 Your Mission

**Goal**: Complete at least 3 successful swaps today

**Recommended order**:
1. ✅ Get testnet ETH (5 min)
2. ✅ Test ETH → Solana (2 min)
3. ✅ Test Bitcoin → Base (10 min with manual deposit)
4. ✅ Test EVM → EVM (2 min)

**Total time**: ~20 minutes

## 🚀 Let's Go!

```bash
# Start here
cargo run --release -- list-swaps

# Then test your first swap
cargo run --release -- test-swap ethereum_sepolia:eth solana_testnet:sol

# Check results
cargo run --release -- history
```

**You got this! 🎉**

---

## 📞 Need Help?

1. Check `docs/CURRENT_STATUS_AND_NEXT_STEPS.md`
2. Check `docs/IMPLEMENTATION_COMPLETE_V2.md`
3. Search `docs/gardenjs.md` for examples
4. Check logs: `RUST_LOG=debug cargo run --release -- test-swap ...`

## 🎊 Success Looks Like

```
═══ Swap Test Result ═══
Pair      : ethereum_sepolia:eth -> solana_testnet:sol
Status    : Completed
Order ID  : f795ecbcb9940fce45f23cf79c6d918d21a605b63f5c84bb8b2124f86b24b70d
Duration  : 45s
Src Init  : 0x1234...
Dst Redeem: 0x5678...

✅ Swap completed successfully!
```

**Now go get those testnet tokens and start swapping! 🚀**
