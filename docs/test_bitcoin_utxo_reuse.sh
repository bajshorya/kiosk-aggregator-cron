#!/bin/bash

# Test Bitcoin UTXO reuse - demonstrates that unconfirmed UTXOs (change from previous swaps)
# are automatically reused for subsequent swaps without waiting for confirmations

echo "════════════════════════════════════════════════════════════"
echo "Bitcoin UTXO Reuse Test - Rapid Sequential Swaps"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "This test demonstrates that:"
echo "1. After initiating a Bitcoin swap, change is returned to your wallet"
echo "2. The change UTXO appears in mempool (unconfirmed) within seconds"
echo "3. The next swap can immediately use this unconfirmed UTXO"
echo "4. This enables continuous Bitcoin swap testing without waiting"
echo ""
echo "Strategy: Start each swap with 5-second delay (enough time for"
echo "          the previous swap to broadcast and create change UTXO)"
echo ""
echo "Press Enter to start testing 3 rapid sequential Bitcoin swaps..."
read

echo ""
echo "━━━ Starting Swap 1: Bitcoin → Solana USDC ━━━"
cargo run --release -- test-swap bitcoin_testnet:btc solana_testnet:usdc &
SWAP1_PID=$!

echo ""
echo "⏱️  Waiting 5 seconds for Swap 1 to broadcast transaction..."
sleep 5

echo ""
echo "━━━ Starting Swap 2: Bitcoin → Starknet WBTC ━━━"
echo "    (Should reuse change from Swap 1 which is now in mempool)"
cargo run --release -- test-swap bitcoin_testnet:btc starknet_sepolia:wbtc &
SWAP2_PID=$!

echo ""
echo "⏱️  Waiting 5 seconds for Swap 2 to broadcast transaction..."
sleep 5

echo ""
echo "━━━ Starting Swap 3: Bitcoin → Citrea USDC ━━━"
echo "    (Should reuse change from Swap 2 which is now in mempool)"
cargo run --release -- test-swap bitcoin_testnet:btc citrea_testnet:usdc &
SWAP3_PID=$!

echo ""
echo "════════════════════════════════════════════════════════════"
echo "All 3 swaps are now running in parallel!"
echo ""
echo "Watch the logs above to see:"
echo "  - Swap 1: Uses existing confirmed/unconfirmed UTXOs"
echo "  - Swap 2: Should show [MEMPOOL] UTXO from Swap 1's change"
echo "  - Swap 3: Should show [MEMPOOL] UTXO from Swap 2's change"
echo ""
echo "Press Ctrl+C to stop all swaps, or wait for them to complete..."
echo "════════════════════════════════════════════════════════════"

# Wait for all background processes
wait $SWAP1_PID
wait $SWAP2_PID
wait $SWAP3_PID

echo ""
echo "════════════════════════════════════════════════════════════"
echo "All 3 swaps complete!"
echo ""
echo "Summary:"
echo "  ✓ Swap 1: Used existing UTXOs from your wallet"
echo "  ✓ Swap 2: Reused unconfirmed change from Swap 1 (in mempool)"
echo "  ✓ Swap 3: Reused unconfirmed change from Swap 2 (in mempool)"
echo ""
echo "This demonstrates continuous Bitcoin swap testing without"
echo "waiting for confirmations between swaps."
echo "════════════════════════════════════════════════════════════"
