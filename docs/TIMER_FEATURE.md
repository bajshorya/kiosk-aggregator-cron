# Timer Feature Documentation

## Overview
The Garden Swap Tester now displays real-time countdown timers in both CLI and Discord to show progress during swap operations.

## Features

### CLI Timer
- Updates every 30 seconds during swap polling
- Shows elapsed time and remaining time
- Displays current poll count
- Format: `⏱️  [swap_pair] Xm Ys elapsed, ~Xm Ys remaining (poll #N)`

### Discord Timer
- Sends periodic status updates as new messages
- Single swap: Updates every 60 seconds (1 minute)
- Batch test: Updates every 120 seconds (2 minutes)
- Shows elapsed and estimated remaining time
- Format: `⏱️ Swap test running... Xm elapsed, ~Xm remaining`

## Implementation Details

### CLI Timer (src/scheduler/runner.rs)

The CLI timer is integrated into the swap polling loop:

```rust
let start_time = Instant::now();
let mut last_timer_update = Instant::now();

loop {
    sleep(poll_every).await;
    poll_count += 1;
    
    // Print timer update every 30 seconds
    let elapsed = start_time.elapsed();
    if elapsed.as_secs() >= last_timer_update.elapsed().as_secs() + 30 {
        let elapsed_secs = elapsed.as_secs();
        let remaining_secs = timeout.as_secs().saturating_sub(elapsed_secs);
        println!(
            "⏱️  [{}] {}m {}s elapsed, ~{}m {}s remaining (poll #{})",
            pair.label(),
            elapsed_secs / 60,
            elapsed_secs % 60,
            remaining_secs / 60,
            remaining_secs % 60,
            poll_count
        );
        last_timer_update = Instant::now();
    }
    
    // ... rest of polling logic
}
```

### Discord Timer (src/discord/commands/test_swap.rs)

The Discord timer runs as a separate async task:

#### Single Swap Timer (60-second intervals)
```rust
let timer_handle = tokio::spawn(async move {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
    interval.tick().await; // Skip first immediate tick
    
    for i in 1..=6 { // Up to 6 minutes
        interval.tick().await;
        let elapsed_mins = i;
        let remaining_mins = 7 - elapsed_mins;
        
        let message = format!("⏱️ Swap test running... {}m elapsed, ~{}m remaining", elapsed_mins, remaining_mins);
        let _ = channel_id.say(&ctx_for_timer.http, message).await;
    }
});
```

#### Batch Test Timer (120-second intervals)
```rust
let timer_handle = tokio::spawn(async move {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(120));
    interval.tick().await; // Skip first immediate tick
    
    for i in 1..=6 { // Up to 12 minutes
        interval.tick().await;
        let elapsed_mins = i * 2;
        let remaining_mins = 14 - elapsed_mins;
        
        let message = format!("⏱️ Batch test running... {}m elapsed, ~{}m remaining", elapsed_mins, remaining_mins);
        let _ = channel_id.say(&ctx_for_timer.http, message).await;
    }
});
```

## Example Output

### CLI Example
```
⏱️  [ethereum_sepolia:eth -> base_sepolia:wbtc] 0m 30s elapsed, ~6m 30s remaining (poll #3)
⏱️  [ethereum_sepolia:eth -> base_sepolia:wbtc] 1m 0s elapsed, ~6m 0s remaining (poll #6)
⏱️  [ethereum_sepolia:eth -> base_sepolia:wbtc] 1m 30s elapsed, ~5m 30s remaining (poll #9)
✅ Swap completed
```

### Discord Example (Single Swap)
```
⏱️ Swap test running... 1m elapsed, ~6m remaining
⏱️ Swap test running... 2m elapsed, ~5m remaining
⏱️ Swap test running... 3m elapsed, ~4m remaining
✅ Cross-Chain Swap: ethereum_sepolia:eth → base_sepolia:wbtc
[Detailed results embed]
```

### Discord Example (Batch Test)
```
⏱️ Batch test running... 2m elapsed, ~12m remaining
⏱️ Batch test running... 4m elapsed, ~10m remaining
⏱️ Batch test running... 6m elapsed, ~8m remaining
🔄 Batch Swap Test Results
[Summary embed]
```

## Timer Behavior

### Early Completion
- If a swap completes before the next timer update, the timer task is cancelled
- Final results are shown immediately
- No unnecessary timer messages after completion

### Timeout
- Timer continues until timeout is reached
- Final timeout message includes actual duration
- Format: `⏰ Timeout (after Xm Ys)`

### Error Handling
- Timer task is always cancelled when operation completes or fails
- Uses `timer_handle.abort()` for cleanup
- Discord timer uses `let _ =` to ignore send failures

## Configuration

### CLI Timer Interval
To change how often the CLI timer updates:

```rust
// Current: 30 seconds
if elapsed.as_secs() >= last_timer_update.elapsed().as_secs() + 30 {
    // Update timer
}

// Example: 60 seconds
if elapsed.as_secs() >= last_timer_update.elapsed().as_secs() + 60 {
    // Update timer
}
```

### Discord Timer Interval

For single swaps:
```rust
// Current: 60 seconds
let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

// Example: 30 seconds
let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
```

For batch tests:
```rust
// Current: 120 seconds (2 minutes)
let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(120));

// Example: 180 seconds (3 minutes)
let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(180));
```

## Benefits

1. **User Awareness**: Users know the operation is progressing
2. **Transparency**: Clear visibility into elapsed and remaining time
3. **Reduced Anxiety**: Users don't wonder if the process is stuck
4. **Better UX**: Real-time feedback improves user experience
5. **Debugging**: Timer output helps identify slow operations

## Rate Limiting Considerations

### Discord
- Single swap: 6 messages max (1 per minute for 6 minutes)
- Batch test: 6 messages max (1 per 2 minutes for 12 minutes)
- Well within Discord's rate limits (5 messages per 5 seconds per channel)

### CLI
- No rate limiting concerns
- Console output is local and instant
- 30-second intervals prevent spam

## Cleanup

Both CLI and Discord timers are properly cleaned up:

### Discord
```rust
// Cancel the timer task when operation completes
timer_handle.abort();
```

### CLI
- Timer is part of the polling loop
- Automatically stops when loop exits
- No explicit cleanup needed

## Future Enhancements

1. **Dynamic Intervals**: Update more frequently as timeout approaches
2. **Progress Percentage**: Show percentage complete
3. **Stage Indicators**: Show which stage of swap is running (quote, submit, poll)
4. **Estimated Completion**: Use historical data to predict completion time
5. **Configurable Intervals**: Allow users to set custom timer intervals
6. **Color Coding**: Use colors in CLI for different time ranges
7. **Sound Alerts**: Optional sound when swap completes (CLI)

## Troubleshooting

### CLI Timer Not Showing
- Check if swap is in polling phase
- Verify 30 seconds have elapsed since last update
- Ensure stdout is not buffered

### Discord Timer Not Updating
- Check Discord API rate limits
- Verify bot has permission to send messages in channel
- Check logs for HTTP errors
- Ensure timer task is spawned correctly

### Timer Shows Wrong Time
- Verify `start_time` is captured at the right moment
- Check timeout duration matches configuration
- Ensure elapsed time calculation is accurate

## Testing

To test the timer feature:

### CLI
```bash
cargo run --release -- test-swap ethereum_sepolia:eth base_sepolia:wbtc
```
Watch for timer updates every 30 seconds in the console.

### Discord
```
/test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc
```
Watch for timer messages every 60 seconds in Discord.

### Batch Test
```
/test-swap-all
```
Watch for timer messages every 2 minutes in Discord.
