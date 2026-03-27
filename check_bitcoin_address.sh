#!/bin/bash
# Quick script to check Bitcoin address and balance

echo "=== Bitcoin Testnet Address Checker ==="
echo ""

# Load from .env
source .env

echo "Checking address: $WALLET_BITCOIN_TESTNET"
echo ""

# Check balance via Blockstream API
echo "Fetching data from Blockstream..."
curl -s "https://blockstream.info/testnet/api/address/$WALLET_BITCOIN_TESTNET" | jq '.'

echo ""
echo "Fetching UTXOs..."
curl -s "https://blockstream.info/testnet/api/address/$WALLET_BITCOIN_TESTNET/utxo" | jq '.'

echo ""
echo "=== Check complete ==="
echo "If you see empty arrays but have funds in a block explorer,"
echo "the address in your .env might not match your actual funded address."
