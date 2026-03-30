#!/bin/bash

# Test script for 3 swaps: BTC, ETH, and SOL
# Tests one swap from each major chain to verify all implementations work

set -e  # Exit on error

echo "════════════════════════════════════════════════════════════"
echo "  Garden Swap Tester - Three Chain Test"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "This script will test 3 swaps sequentially:"
echo "  1. Bitcoin Testnet → Solana USDC"
echo "  2. Ethereum Sepolia → Solana SOL"
echo "  3. Solana Devnet → Ethereum ETH"
echo ""
echo "Each swap will run for up to 30 minutes (1800 seconds)"
echo ""
read -p "Press Enter to start testing..."

# Ensure we're in the right directory
cd "$(dirname "$0")"

# Build the project
echo ""
echo "━━━ Building project... ━━━"
cargo build --release

# Test 1: Bitcoin → Solana USDC
echo ""
echo "════════════════════════════════════════════════════════════"
echo "  TEST 1/3: Bitcoin Testnet → Solana USDC"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Amount: 50,000 sats (~$50)"
echo "Expected: Transaction built, signed, and broadcasted"
echo "Garden should detect: src_init populated"
echo ""

cargo run --release -- test-swap bitcoin_testnet:btc solana_testnet:usdc

echo ""
echo "✅ Test 1 complete!"
echo ""
read -p "Press Enter to continue to Test 2..."

# Test 2: Ethereum → Solana SOL
echo ""
echo "════════════════════════════════════════════════════════════"
echo "  TEST 2/3: Ethereum Sepolia → Solana SOL"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Amount: 0.005 ETH (~$10)"
echo "Expected: Transaction sent to mempool and confirmed"
echo "Note: Garden's ETH indexer may be slow/broken"
echo ""

cargo run --release -- test-swap ethereum_sepolia:eth solana_testnet:sol

echo ""
echo "✅ Test 2 complete!"
echo ""
read -p "Press Enter to continue to Test 3..."

# Test 3: Solana → Ethereum ETH
echo ""
echo "════════════════════════════════════════════════════════════"
echo "  TEST 3/3: Solana Devnet → Ethereum Sepolia ETH"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Amount: 0.1 SOL (~$8)"
echo "Expected: Gasless transaction signed and submitted"
echo "This swap typically works reliably"
echo ""

cargo run --release -- test-swap solana_testnet:sol ethereum_sepolia:eth

echo ""
echo "✅ Test 3 complete!"
echo ""

# Summary
echo ""
echo "════════════════════════════════════════════════════════════"
echo "  All Tests Complete!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Check your results:"
echo "  • Database: garden_swaps.db"
echo "  • View history: cargo run --release -- history"
echo "  • Garden dashboard: https://testnet.garden.finance"
echo ""
echo "Expected results:"
echo "  ✅ Bitcoin swap: Should show 'In progress' on Garden"
echo "  ⚠️  Ethereum swap: May timeout (Garden indexer issue)"
echo "  ✅ Solana swap: Should complete successfully"
echo ""
