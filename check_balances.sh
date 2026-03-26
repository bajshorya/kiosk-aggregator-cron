#!/bin/bash

WALLET="0x55D246A6c46C4371ffcb6e18cD1D10Eca2d6F406"

echo "Checking balances for wallet: $WALLET"
echo ""

echo "=== Ethereum Sepolia ==="
curl -s -X POST https://ethereum-sepolia-rpc.publicnode.com \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$WALLET\",\"latest\"],\"id\":1}" | jq -r '.result' | xargs printf "%d\n" | awk '{printf "Balance: %.6f ETH\n", $1/1000000000000000000}'
echo ""

echo "=== Base Sepolia ==="
curl -s -X POST https://sepolia.base.org \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$WALLET\",\"latest\"],\"id\":1}" | jq -r '.result' | xargs printf "%d\n" | awk '{printf "Balance: %.6f ETH\n", $1/1000000000000000000}'
echo ""

echo "=== Arbitrum Sepolia ==="
curl -s -X POST https://sepolia-rollup.arbitrum.io/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$WALLET\",\"latest\"],\"id\":1}" | jq -r '.result' | xargs printf "%d\n" | awk '{printf "Balance: %.6f ETH\n", $1/1000000000000000000}'
