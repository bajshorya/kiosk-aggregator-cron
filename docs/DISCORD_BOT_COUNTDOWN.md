# Discord Bot Countdown Timer Feature - DEPRECATED

## Status: REMOVED

The countdown timer feature has been removed due to Discord webhook token errors when trying to edit deferred interaction responses.

## Issue Encountered

When attempting to edit the deferred response using the raw HTTP API, Discord returns "Invalid Webhook Token" errors. This is because:

1. Poise manages interaction tokens internally
2. Direct HTTP API calls conflict with poise's interaction handling
3. Deferred responses have specific token lifecycles that can't be easily accessed

## Alternative Approach Considered

We attempted to use `http.edit_original_interaction_response()` but this caused:
- "Invalid Webhook Token" errors
- "Error while handling error" cascading failures
- Unreliable message updates

## Current Implementation

The bot now uses a simpler approach:
- Defers the response immediately
- Processes the swap test with timeout (10 min single, 20 min batch)
- Returns the final result when complete
- No intermediate countdown updates

## Why This Is Acceptable

1. **Timeout Protection**: Operations still have proper timeouts
2. **Clean Execution**: No webhook token errors
3. **Reliable Results**: Final results are always delivered
4. **Simpler Code**: Less complexity, fewer failure points

## Future Alternatives

If countdown functionality is needed in the future, consider:

1. **Separate Messages**: Send new messages instead of editing (but creates spam)
2. **Ephemeral Updates**: Use ephemeral messages for progress (only visible to user)
3. **Bot Status**: Update bot's status/presence instead of messages
4. **Webhook Approach**: Use a separate webhook for updates
5. **Polling Command**: Add a `/status` command to check progress

## Recommendation

For now, the simple approach without countdown is the most reliable. Users can trust that:
- Single swaps complete within 10 minutes
- Batch tests complete within 20 minutes
- They'll receive results when done
- Timeouts are handled gracefully

## Documentation Status

This file is kept for historical reference. The countdown feature is not currently implemented.
