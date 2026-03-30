#!/bin/bash

# Balance Checker for Garden Swap Tester
# Checks all token balances across chains

WALLET="0x55D246A6c46C4371ffcb6e18cD1D10Eca2d6F406"
BTC_WALLET="tb1qypyl87pagnmcted8d8gnmxxps7dyqstllhhe9z"
SOLANA_WALLET="5ymiDmUNjH1ehfVaR9W95kNXyKuaLsVPGBS7dXLP41ny"

OUTPUT_FILE="balances.txt"

echo "=== Token Balance Report ===" > $OUTPUT_FILE
echo "Generated: $(date)" >> $OUTPUT_FILE
echo "" >> $OUTPUT_FILE

# Ethereum Sepolia
echo "━━━ ETHEREUM SEPOLIA ━━━" >> $OUTPUT_FILE
ETH_SEPOLIA=$(cast balance $WALLET --rpc-url https://ethereum-sepolia-rpc.publicnode.com 2>/dev/null || echo "0")
echo "ETH: $(cast --from-wei $ETH_SEPOLIA 2>/dev/null || echo "Error")" >> $OUTPUT_FILE

WBTC_ETH=$(cast call 0x29f2D40B0605204364af54EC677bD022dA425d03 "balanceOf(address)(uint256)" $WALLET --rpc-url https://ethereum-sepolia-rpc.publicnode.com 2>/dev/null || echo "0")
echo "WBTC: $(echo "scale=8; $WBTC_ETH / 100000000" | bc 2>/dev/null || echo "0")" >> $OUTPUT_FILE

USDC_ETH=$(cast call 0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238 "balanceOf(address)(uint256)" $WALLET --rpc-url https://ethereum-sepolia-rpc.publicnode.com 2>/dev/null || echo "0")
echo "USDC: $(echo "scale=6; $USDC_ETH / 1000000" | bc 2>/dev/null || echo "0")" >> $OUTPUT_FILE
echo "" >> $OUTPUT_FILE

# Base Sepolia
echo "━━━ BASE SEPOLIA ━━━" >> $OUTPUT_FILE
ETH_BASE=$(cast balance $WALLET --rpc-url https://sepolia.base.org 2>/dev/null || echo "0")
echo "ETH: $(cast --from-wei $ETH_BASE 2>/dev/null || echo "Error")" >> $OUTPUT_FILE

USDC_BASE=$(cast call 0x730be401ef981d199a0560c87dfddafd3ec1c493 "balanceOf(address)(uint256)" $WALLET --rpc-url https://sepolia.base.org 2>/dev/null || echo "0")
echo "USDC: $(echo "scale=6; $USDC_BASE / 1000000" | bc 2>/dev/null || echo "0")" >> $OUTPUT_FILE

CBLTC_BASE=$(cast call 0x7a7dbf1e9f0d2dc4e5f2c1c5b8e3f4a5b6c7d8e9 "balanceOf(address)(uint256)" $WALLET --rpc-url https://sepolia.base.org 2>/dev/null || echo "0")
echo "cbLTC: $(echo "scale=8; $CBLTC_BASE / 100000000" | bc 2>/dev/null || echo "0")" >> $OUTPUT_FILE
echo "" >> $OUTPUT_FILE

# Arbitrum Sepolia
echo "━━━ ARBITRUM SEPOLIA ━━━" >> $OUTPUT_FILE
ETH_ARB=$(cast balance $WALLET --rpc-url https://sepolia-rollup.arbitrum.io/rpc 2>/dev/null || echo "0")
echo "ETH: $(cast --from-wei $ETH_ARB 2>/dev/null || echo "Error")" >> $OUTPUT_FILE

WBTC_ARB=$(cast call 0xb5ae9785349186069c48794a763db39ec756b1cf "balanceOf(address)(uint256)" $WALLET --rpc-url https://sepolia-rollup.arbitrum.io/rpc 2>/dev/null || echo "0")
echo "WBTC: $(echo "scale=8; $WBTC_ARB / 100000000" | bc 2>/dev/null || echo "0")" >> $OUTPUT_FILE

USDC_ARB=$(cast call 0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d "balanceOf(address)(uint256)" $WALLET --rpc-url https://sepolia-rollup.arbitrum.io/rpc 2>/dev/null || echo "0")
echo "USDC: $(echo "scale=6; $USDC_ARB / 1000000" | bc 2>/dev/null || echo "0")" >> $OUTPUT_FILE
echo "" >> $OUTPUT_FILE

# Bitcoin Testnet4
echo "━━━ BITCOIN TESTNET4 ━━━" >> $OUTPUT_FILE
BTC_DATA=$(curl -s "https://mempool.space/testnet4/api/address/$BTC_WALLET")
BTC_BALANCE=$(echo "$BTC_DATA" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data['chain_stats']['funded_txo_sum'])" 2>/dev/null || echo "0")
BTC_SPENT=$(echo "$BTC_DATA" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data['chain_stats']['spent_txo_sum'])" 2>/dev/null || echo "0")
BTC_FINAL=$((BTC_BALANCE - BTC_SPENT))
echo "BTC: $(echo "scale=8; $BTC_FINAL / 100000000" | bc 2>/dev/null || echo "0.00000000") (${BTC_FINAL} sats)" >> $OUTPUT_FILE
echo "" >> $OUTPUT_FILE

# Solana Devnet
echo "━━━ SOLANA DEVNET ━━━" >> $OUTPUT_FILE
SOL_DATA=$(curl -s https://api.devnet.solana.com -X POST -H "Content-Type: application/json" -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getBalance\",\"params\":[\"$SOLANA_WALLET\"]}")
SOL_BALANCE=$(echo "$SOL_DATA" | python3 -c "import sys, json; data=json.load(sys.stdin); print(data['result']['value'])" 2>/dev/null || echo "0")
echo "SOL: $(echo "scale=9; $SOL_BALANCE / 1000000000" | bc 2>/dev/null || echo "0.000000000")" >> $OUTPUT_FILE

# Solana USDC (token account)
USDC_SOL_DATA=$(curl -s https://api.devnet.solana.com -X POST -H "Content-Type: application/json" -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getTokenAccountsByOwner\",\"params\":[\"$SOLANA_WALLET\",{\"mint\":\"Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr\"},{\"encoding\":\"jsonParsed\"}]}")
USDC_SOL=$(echo "$USDC_SOL_DATA" | python3 -c "import sys, json; data=json.load(sys.stdin); accounts=data.get('result',{}).get('value',[]); print(accounts[0]['account']['data']['parsed']['info']['tokenAmount']['uiAmount'] if accounts else 0)" 2>/dev/null || echo "0")
echo "USDC: $USDC_SOL" >> $OUTPUT_FILE
echo "" >> $OUTPUT_FILE

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━" >> $OUTPUT_FILE
echo "Report saved to: $OUTPUT_FILE" >> $OUTPUT_FILE

# Also print to console
cat $OUTPUT_FILE

echo ""
echo "✅ Balance report saved to: $OUTPUT_FILE"
