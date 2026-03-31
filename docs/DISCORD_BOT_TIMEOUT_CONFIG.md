# Discord Bot Timeout Configuration

## Overview
The Discord bot has configurable timeouts to ensure operations don't hang indefinitely and provide timely feedback to users.

## Timeout Values

### Single Swap Test (`/test-swap`)
- **Timeout**: 7 minutes (420 seconds)
- **Purpose**: Ensures individual swap tests complete within a reasonable timeframe
- **Behavior**: If a swap takes longer than 7 minutes, it will be cancelled and the user will be notified
- **Note**: The swap might still complete on-chain even after timeout
- **Early Completion**: If the swap completes in less than 7 minutes, results are returned immediately

### Batch Swap Test (`/test-swap-all`)
- **Timeout**: 14 minutes (840 seconds)
- **Purpose**: Allows sufficient time for all configured swap pairs to be tested
- **Behavior**: If the batch test takes longer than 14 minutes, it will be cancelled
- **Note**: Some swaps might still complete on-chain even after timeout
- **Early Completion**: If all swaps complete in less than 14 minutes, results are returned immediately

## Implementation Details

### Code Location
File: `src/discord/commands/test_swap.rs`

### Single Swap Timeout
```rust
// In test_swap() function
let timeout_duration = tokio::time::Duration::from_secs(420); // 7 minutes

let result = tokio::time::timeout(timeout_duration, async {
    stdout_reader.read_to_string(&mut stdout_content).await?;
    stderr_reader.read_to_string(&mut stderr_content).await?;
    child.wait().await
}).await;
```

### Batch Test Timeout
```rust
// In test_swap_all() function
let timeout_duration = tokio::time::Duration::from_secs(840); // 14 minutes

let result = tokio::time::timeout(timeout_duration, async {
    stdout_reader.read_to_string(&mut stdout_content).await?;
    stderr_reader.read_to_string(&mut stderr_content).await?;
    child.wait().await
}).await;
```

### Early Completion
The implementation uses `tokio::time::timeout()` which returns as soon as the inner async block completes. This means:
- If a swap finishes in 2 minutes, you get results in 2 minutes (not 7)
- If batch tests finish in 8 minutes, you get results in 8 minutes (not 14)
- The timeout is only a maximum limit, not a fixed wait time

## Error Handling

### Timeout Error Message (Single Swap)
```
⏰ Timeout

The swap test took longer than 7 minutes and was cancelled. 
The swap might still complete on-chain.
```

### Timeout Error Message (Batch Test)
```
⏰ Timeout

The batch test took longer than 14 minutes and was cancelled. 
Some swaps might still complete on-chain.
```

### Success Message (Early Completion)
When swaps complete before the timeout, the bot immediately returns the results with:
- Detailed swap information in a rich embed
- Transaction hashes and addresses
- Completion status with appropriate emoji (✅, ❌, ⏰, ↩️)
- Duration of the actual swap

## Adjusting Timeouts

If you need to adjust the timeout values:

1. Open `src/discord/commands/test_swap.rs`
2. Find the `test_swap()` function for single swap timeout
3. Find the `test_swap_all()` function for batch test timeout
4. Modify the `from_secs()` value:
   - Current single: `from_secs(420)` = 7 minutes
   - Current batch: `from_secs(840)` = 14 minutes
   - Example: `from_secs(600)` = 10 minutes
5. Update the error message to reflect the new timeout
6. Rebuild the project: `cargo build --release`

## Recommendations

### When to Increase Timeouts
- Network congestion is causing slower transaction confirmations
- Testing on chains with longer block times
- Running more swap pairs in batch mode
- Testing with larger amounts that require more confirmations

### When to Decrease Timeouts
- Testing on faster chains (e.g., Solana)
- Need quicker feedback for failed swaps
- Running fewer swap pairs in batch mode
- Development/testing environment with faster iterations

## Process Management

The bot uses `kill_on_drop(true)` to ensure child processes are properly terminated when timeouts occur:

```rust
let mut child = Command::new(&command)
    .args(&final_args)
    .current_dir(&project_root)
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .kill_on_drop(true) // Ensures cleanup on timeout
    .spawn()?;
```

This prevents zombie processes and ensures clean resource cleanup.

## Monitoring

To monitor timeout occurrences:

1. Check Discord bot logs for timeout messages
2. Look for "⏰ Timeout" messages in Discord channels
3. Review the tracing logs for timeout events
4. Monitor on-chain transactions to see if they completed despite timeout

## Best Practices

1. **Set realistic timeouts**: Based on average swap completion times (currently 7 min single, 14 min batch)
2. **Monitor timeout frequency**: If timeouts are common, increase the duration
3. **Inform users**: Timeout messages should be clear about what happened
4. **Check on-chain**: Remind users that swaps might still complete
5. **Log timeout events**: For debugging and optimization
6. **Early returns**: The bot returns results immediately when swaps complete early

## Future Enhancements

1. **Dynamic timeouts**: Adjust based on chain congestion
2. **Configurable timeouts**: Allow users to set custom timeout values
3. **Timeout warnings**: Send intermediate updates before timeout
4. **Retry mechanism**: Automatically retry timed-out operations
5. **Timeout analytics**: Track and report timeout statistics
