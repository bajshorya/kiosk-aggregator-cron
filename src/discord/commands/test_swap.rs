use poise::serenity_prelude::{CreateAttachment, CreateEmbed, CreateEmbedFooter, Colour};
use std::env;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tracing::info;

use crate::discord::bot::{Context, Error};

const DISCORD_MESSAGE_LIMIT: usize = 2000;

/// Health check command - replies with pong
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("pong 🏓").await?;
    Ok(())
}

/// Test a specific swap pair
///
/// Example: /test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc amount:1000000
#[poise::command(slash_command, prefix_command, rename = "test-swap")]
pub async fn test_swap(
    ctx: Context<'_>,
    #[description = "From asset (e.g., ethereum_sepolia:eth)"]
    #[autocomplete = "autocomplete_asset"]
    from_asset: String,
    #[description = "To asset (e.g., base_sepolia:wbtc)"]
    #[autocomplete = "autocomplete_asset"]
    to_asset: String,
    #[description = "Amount in smallest units (optional)"] 
    amount: Option<String>,
) -> Result<(), Error> {
    info!(
        "Received /test-swap command: {} -> {} (amount: {:?})",
        from_asset, to_asset, amount
    );

    // Validate the swap pair before executing
    if !is_valid_swap_pair(&from_asset, &to_asset) {
        let error_msg = format!(
            "❌ **Invalid Swap Pair**\n\n\
            The swap from `{}` to `{}` is not supported.\n\n\
            **Possible issues:**\n\
            • Assets must be on different chains\n\
            • Both assets must be in the supported list\n\
            • Check the asset format: `chain:token` (e.g., `ethereum_sepolia:eth`)\n\n\
            Use the autocomplete dropdown to see valid options.",
            from_asset, to_asset
        );
        ctx.say(error_msg).await?;
        return Ok(());
    }

    // Defer the response immediately to prevent timeout
    ctx.defer().await?;

    // Get the project root directory (where Cargo.toml is located)
    let project_root = env::current_dir()?;

    // Use the pre-built binary directly instead of cargo run for speed
    let binary_path = project_root.join("target").join("release").join("kiosk-aggregator-cron.exe");
    
    let (command, args) = if binary_path.exists() {
        // Use pre-built release binary (much faster)
        info!("Using pre-built binary: {:?}", binary_path);
        (binary_path.to_string_lossy().to_string(), vec!["test-swap".to_string(), from_asset.clone(), to_asset.clone()])
    } else {
        // Fall back to cargo run if binary doesn't exist
        info!("Binary not found, using cargo run (will be slower)");
        ("cargo".to_string(), vec!["run".to_string(), "--release".to_string(), "--".to_string(), "test-swap".to_string(), from_asset.clone(), to_asset.clone()])
    };
    
    let mut final_args = args;
    
    // Add amount if provided
    if let Some(amt) = amount {
        final_args.push(amt);
    }

    info!("Executing: {} {:?} in {:?}", command, final_args, project_root);

    // Spawn the command asynchronously with timeout handling
    let mut child = Command::new(&command)
        .args(&final_args)
        .current_dir(&project_root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true) // Ensure child process is killed if dropped
        .spawn()?;

    // Capture stdout and stderr
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    let mut stdout_reader = tokio::io::BufReader::new(stdout);
    let mut stderr_reader = tokio::io::BufReader::new(stderr);

    // Read all output with timeout (7 minutes max)
    let timeout_duration = tokio::time::Duration::from_secs(420); // 7 minutes
    let start_time = std::time::Instant::now();
    
    info!("Starting timer task for single swap (7 min timeout)");
    
    // Spawn a timer task to send periodic updates to Discord
    let ctx_for_timer = ctx.serenity_context().clone();
    let channel_id = ctx.channel_id();
    let timer_handle = tokio::spawn(async move {
        // Send initial message immediately
        let _ = channel_id.say(&ctx_for_timer.http, "⏱️ Swap test started... (max 7 minutes)").await;
        
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60)); // Update every 60 seconds
        interval.tick().await; // Skip first immediate tick
        
        for i in 1..=6 { // Up to 6 minutes (7 min timeout - 1 min buffer)
            interval.tick().await;
            let elapsed_mins = i;
            let remaining_mins = 7 - elapsed_mins;
            
            let message = format!("⏱️ Swap test running... {}m elapsed, ~{}m remaining", elapsed_mins, remaining_mins);
            let _ = channel_id.say(&ctx_for_timer.http, message).await;
        }
    });
    
    // Spawn tasks to read stdout and stderr concurrently
    let stdout_handle = tokio::spawn(async move {
        let mut content = String::new();
        let _ = stdout_reader.read_to_string(&mut content).await;
        content
    });
    
    let stderr_handle = tokio::spawn(async move {
        let mut content = String::new();
        let _ = stderr_reader.read_to_string(&mut content).await;
        content
    });
    
    // Wait for process with timeout
    let wait_result = tokio::time::timeout(timeout_duration, child.wait()).await;

    // Cancel the timer task
    timer_handle.abort();
    
    // Always try to get the output, even if timeout occurred
    let stdout_content = stdout_handle.await.unwrap_or_default();
    let stderr_content = stderr_handle.await.unwrap_or_default();
    
    // Calculate actual duration
    let actual_duration = start_time.elapsed();
    let duration_secs = actual_duration.as_secs();
    let duration_msg = if duration_secs < 60 {
        format!("{}s", duration_secs)
    } else {
        format!("{}m {}s", duration_secs / 60, duration_secs % 60)
    };

    let status = match wait_result {
        Ok(Ok(status)) => status,
        Ok(Err(e)) => {
            let error_msg = format!("❌ **Process Error** (after {})\n\nFailed to execute command: {}", duration_msg, e);
            ctx.send(poise::CreateReply::default().content(error_msg).ephemeral(false)).await?;
            
            // Still try to show output if we have any
            if !stdout_content.is_empty() || !stderr_content.is_empty() {
                let filtered_stderr = filter_compilation_output(&stderr_content);
                let mut output = String::new();
                if !stdout_content.is_empty() {
                    output.push_str(&stdout_content);
                }
                if !filtered_stderr.is_empty() {
                    if !output.is_empty() {
                        output.push_str("\n\n");
                    }
                    output.push_str(&filtered_stderr);
                }
                if !output.is_empty() {
                    send_output(ctx, &output).await?;
                }
            }
            
            return Ok(());
        }
        Err(_) => {
            // Timeout occurred - show whatever output we captured
            info!("Single swap timed out, captured {} bytes stdout, {} bytes stderr", stdout_content.len(), stderr_content.len());
            
            let filtered_stderr = filter_compilation_output(&stderr_content);
            let mut output = String::new();
            if !stdout_content.is_empty() {
                output.push_str(&stdout_content);
            }
            if !filtered_stderr.is_empty() {
                if !output.is_empty() {
                    output.push_str("\n\n");
                }
                output.push_str(&filtered_stderr);
            }
            
            // Send timeout message with duration - use channel directly to avoid webhook token expiry
            let timeout_msg = format!("⏰ **Timeout** (after {})\n\nThe swap test took longer than 7 minutes and was cancelled. Showing partial results below.\n\nThe swap might still complete on-chain.", duration_msg);
            let _ = ctx.channel_id().say(&ctx.serenity_context().http, timeout_msg).await;
            
            // Try to send whatever output we captured - use channel directly
            if !output.is_empty() {
                // Try to parse and format the output as an embed
                if let Some(embed) = parse_swap_output_to_embed(&output) {
                    let _ = ctx.channel_id().send_message(&ctx.serenity_context().http, 
                        serenity::all::CreateMessage::new().embed(embed)
                    ).await;
                } else if output.len() <= DISCORD_MESSAGE_LIMIT {
                    let _ = ctx.channel_id().say(&ctx.serenity_context().http, &output).await;
                } else {
                    // Split into chunks
                    let chunks = split_into_chunks(&output, DISCORD_MESSAGE_LIMIT);
                    for chunk in chunks.iter().take(5) {
                        let _ = ctx.channel_id().say(&ctx.serenity_context().http, chunk).await;
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await; // Rate limit protection
                    }
                }
            } else {
                let _ = ctx.channel_id().say(&ctx.serenity_context().http, "⚠️ No output captured before timeout. The process may have been waiting for input or stuck.").await;
            }
            
            return Ok(());
        }
    };

    // Filter out compilation messages from stderr
    let filtered_stderr = filter_compilation_output(&stderr_content);

    // Combine stdout and filtered stderr
    let mut output = String::new();
    if !stdout_content.is_empty() {
        output.push_str("**Output:**\n```\n");
        output.push_str(&stdout_content);
        output.push_str("\n```\n");
    }
    
    // Only include stderr if there are actual errors (not just compilation info)
    if !filtered_stderr.is_empty() {
        output.push_str("**Errors/Warnings:**\n```\n");
        output.push_str(&filtered_stderr);
        output.push_str("\n```\n");
    }

    // Add status code
    let status_msg = if status.success() {
        format!("✅ Command completed successfully")
    } else {
        format!(
            "❌ Command failed (exit code: {})",
            status.code().unwrap_or(-1)
        )
    };
    output.push_str(&format!("\n{}", status_msg));

    // Send the output back to Discord
    send_output(ctx, &output).await?;

    Ok(())
}

/// Run all swap tests in batch mode
///
/// Executes: cargo run -- test-swap run-once
#[poise::command(slash_command, prefix_command, rename = "test-swap-all")]
pub async fn test_swap_all(ctx: Context<'_>) -> Result<(), Error> {
    info!("Received /test-swap-all command");

    // Defer the response immediately to prevent timeout
    ctx.defer().await?;

    // Get the project root directory (where Cargo.toml is located)
    let project_root = env::current_dir()?;

    // Use the pre-built binary directly instead of cargo run for speed
    let binary_path = project_root.join("target").join("release").join("kiosk-aggregator-cron.exe");
    
    let (command, args) = if binary_path.exists() {
        // Use pre-built release binary (much faster)
        info!("Using pre-built binary for batch test: {:?}", binary_path);
        (binary_path.to_string_lossy().to_string(), vec!["run-once".to_string()])
    } else {
        // Fall back to cargo run if binary doesn't exist
        info!("Binary not found, using cargo run (will be slower)");
        ("cargo".to_string(), vec!["run".to_string(), "--release".to_string(), "--".to_string(), "run-once".to_string()])
    };

    info!("Executing: {} {:?} in {:?}", command, args, project_root);

    // Spawn the command asynchronously
    let mut child = Command::new(&command)
        .args(&args)
        .current_dir(&project_root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    // Capture stdout and stderr
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    let mut stdout_reader = tokio::io::BufReader::new(stdout);
    let mut stderr_reader = tokio::io::BufReader::new(stderr);

    // Read all output with timeout (14 minutes for batch operations)
    let timeout_duration = tokio::time::Duration::from_secs(840); // 14 minutes
    let start_time = std::time::Instant::now();
    
    info!("Starting timer task for batch test (14 min timeout)");
    
    // Spawn a timer task to send periodic updates to Discord
    let ctx_for_timer = ctx.serenity_context().clone();
    let channel_id = ctx.channel_id();
    let timer_handle = tokio::spawn(async move {
        // Send initial message immediately
        let _ = channel_id.say(&ctx_for_timer.http, "⏱️ Batch test started... (max 14 minutes)").await;
        
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(120)); // Update every 2 minutes
        interval.tick().await; // Skip first immediate tick
        
        for i in 1..=6 { // Up to 12 minutes (14 min timeout - 2 min buffer)
            interval.tick().await;
            let elapsed_mins = i * 2;
            let remaining_mins = 14 - elapsed_mins;
            
            let message = format!("⏱️ Batch test running... {}m elapsed, ~{}m remaining", elapsed_mins, remaining_mins);
            let _ = channel_id.say(&ctx_for_timer.http, message).await;
        }
    });
    
    // Spawn tasks to read stdout and stderr concurrently
    let stdout_handle = tokio::spawn(async move {
        let mut content = String::new();
        let _ = stdout_reader.read_to_string(&mut content).await;
        content
    });
    
    let stderr_handle = tokio::spawn(async move {
        let mut content = String::new();
        let _ = stderr_reader.read_to_string(&mut content).await;
        content
    });
    
    // Wait for process with timeout
    let wait_result = tokio::time::timeout(timeout_duration, child.wait()).await;

    // Cancel the timer task
    timer_handle.abort();
    
    // Always try to get the output, even if timeout occurred
    let stdout_content = stdout_handle.await.unwrap_or_default();
    let stderr_content = stderr_handle.await.unwrap_or_default();
    
    // Calculate actual duration
    let actual_duration = start_time.elapsed();
    let duration_secs = actual_duration.as_secs();
    let duration_msg = if duration_secs < 60 {
        format!("{}s", duration_secs)
    } else {
        format!("{}m {}s", duration_secs / 60, duration_secs % 60)
    };

    let status = match wait_result {
        Ok(Ok(status)) => status,
        Ok(Err(e)) => {
            let error_msg = format!("❌ **Process Error** (after {})\n\nFailed to execute batch test: {}", duration_msg, e);
            ctx.send(poise::CreateReply::default().content(error_msg).ephemeral(false)).await?;
            
            // Still try to show output if we have any
            if !stdout_content.is_empty() || !stderr_content.is_empty() {
                let filtered_stderr = filter_compilation_output(&stderr_content);
                let mut output = String::new();
                if !stdout_content.is_empty() {
                    output.push_str(&stdout_content);
                }
                if !filtered_stderr.is_empty() {
                    if !output.is_empty() {
                        output.push_str("\n\n");
                    }
                    output.push_str(&filtered_stderr);
                }
                if !output.is_empty() {
                    send_output(ctx, &output).await?;
                }
            }
            
            return Ok(());
        }
        Err(_) => {
            // Timeout occurred - show whatever output we captured
            info!("Batch test timed out, captured {} bytes stdout, {} bytes stderr", stdout_content.len(), stderr_content.len());
            
            let filtered_stderr = filter_compilation_output(&stderr_content);
            let mut output = String::new();
            if !stdout_content.is_empty() {
                output.push_str(&stdout_content);
            }
            if !filtered_stderr.is_empty() {
                if !output.is_empty() {
                    output.push_str("\n\n");
                }
                output.push_str(&filtered_stderr);
            }
            
            // Send timeout message with duration - use channel directly to avoid webhook token expiry
            let timeout_msg = format!("⏰ **Timeout** (after {})\n\nThe batch test took longer than 14 minutes and was cancelled. Showing partial results below.\n\nSome swaps might still complete on-chain.", duration_msg);
            let _ = ctx.channel_id().say(&ctx.serenity_context().http, timeout_msg).await;
            
            // Try to send whatever output we captured - use channel directly
            if !output.is_empty() {
                // Try to parse and format the output as an embed
                if let Some(embed) = parse_swap_output_to_embed(&output) {
                    let _ = ctx.channel_id().send_message(&ctx.serenity_context().http, 
                        serenity::all::CreateMessage::new().embed(embed)
                    ).await;
                } else if output.len() <= DISCORD_MESSAGE_LIMIT {
                    let _ = ctx.channel_id().say(&ctx.serenity_context().http, &output).await;
                } else {
                    // Split into chunks or send as file
                    let chunks = split_into_chunks(&output, DISCORD_MESSAGE_LIMIT);
                    for chunk in chunks.iter().take(5) {
                        let _ = ctx.channel_id().say(&ctx.serenity_context().http, chunk).await;
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await; // Rate limit protection
                    }
                }
            } else {
                let _ = ctx.channel_id().say(&ctx.serenity_context().http, "⚠️ No output captured before timeout. The process may have been waiting for input or stuck.").await;
            }
            
            return Ok(());
        }
    };

    // Filter out compilation messages from stderr
    let filtered_stderr = filter_compilation_output(&stderr_content);

    // Combine stdout and filtered stderr
    let mut output = String::new();
    if !stdout_content.is_empty() {
        output.push_str("**Output:**\n```\n");
        output.push_str(&stdout_content);
        output.push_str("\n```\n");
    }
    
    // Only include stderr if there are actual errors (not just compilation info)
    if !filtered_stderr.is_empty() {
        output.push_str("**Errors/Warnings:**\n```\n");
        output.push_str(&filtered_stderr);
        output.push_str("\n```\n");
    }

    // Add status code
    let status_msg = if status.success() {
        format!("✅ All swap tests completed successfully")
    } else {
        format!(
            "❌ Some swap tests failed (exit code: {})",
            status.code().unwrap_or(-1)
        )
    };
    output.push_str(&format!("\n{}", status_msg));

    // Send the output back to Discord
    send_output(ctx, &output).await?;

    Ok(())
}

/// List all available swap pairs
///
/// Shows all supported swap pairs with their chains and tokens
#[poise::command(slash_command, prefix_command, rename = "list-swaps")]
pub async fn list_swaps(ctx: Context<'_>) -> Result<(), Error> {
    info!("Received /list-swaps command");

    // Get all supported assets
    let assets = get_supported_assets();
    
    // Group assets by chain
    let mut chains: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    
    for asset in assets {
        let parts: Vec<&str> = asset.split(':').collect();
        if parts.len() == 2 {
            let chain = parts[0].to_string();
            let token = parts[1].to_string();
            chains.entry(chain).or_insert_with(Vec::new).push(token);
        }
    }
    
    // Create embed with all swap pairs
    let mut embed = CreateEmbed::default()
        .title("🔄 Available Swap Pairs")
        .description("All supported chains and tokens for cross-chain swaps")
        .colour(Colour::from_rgb(52, 152, 219)) // Blue
        .timestamp(serenity::all::Timestamp::now());
    
    // Sort chains alphabetically
    let mut chain_names: Vec<String> = chains.keys().cloned().collect();
    chain_names.sort();
    
    // Add fields for each chain
    for chain_name in chain_names {
        if let Some(tokens) = chains.get(&chain_name) {
            let formatted_chain = chain_name
                .replace("_sepolia", " (Sepolia)")
                .replace("_testnet", " (Testnet)")
                .replace("_shasta", " (Shasta)")
                .replace("_", " ")
                .split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            
            let token_list = tokens
                .iter()
                .map(|t| format!("`{}`", t.to_uppercase()))
                .collect::<Vec<_>>()
                .join(", ");
            
            embed = embed.field(formatted_chain, token_list, false);
        }
    }
    
    // Add footer with usage instructions
    embed = embed.footer(CreateEmbedFooter::new(
        "💡 Use /test-swap to test a swap between any two assets on different chains"
    ));
    
    // Add example field
    let example_text = "```\n/test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc\n```";
    embed = embed.field("📝 Example Usage", example_text, false);
    
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    
    Ok(())
}

/// Helper function to send output to Discord, handling the 2000 character limit
async fn send_output(ctx: Context<'_>, output: &str) -> Result<(), Error> {
    // Try to parse and format the output as an embed
    if let Some(embed) = parse_swap_output_to_embed(output) {
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
    } else if output.len() <= DISCORD_MESSAGE_LIMIT {
        // Output fits in a single message
        ctx.say(output).await?;
    } else if output.len() <= DISCORD_MESSAGE_LIMIT * 5 {
        // Split into multiple messages (up to 5 messages = 10,000 chars)
        let chunks = split_into_chunks(output, DISCORD_MESSAGE_LIMIT);
        for (i, chunk) in chunks.iter().enumerate() {
            if i == 0 {
                ctx.say(chunk).await?;
            } else {
                ctx.channel_id().say(&ctx, chunk).await?;
            }
        }
    } else {
        // Output is too large, send as a file attachment
        info!("Output too large ({} bytes), sending as file", output.len());

        let attachment = CreateAttachment::bytes(output.as_bytes().to_vec(), "output.txt");

        ctx.send(
            poise::CreateReply::default()
                .content("📄 Output is too large, attached as file:")
                .attachment(attachment),
        )
        .await?;
    }

    Ok(())
}

/// Split a string into chunks that fit within Discord's message limit
fn split_into_chunks(text: &str, max_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();

    for line in text.lines() {
        // If adding this line would exceed the limit, start a new chunk
        if current_chunk.len() + line.len() + 1 > max_size {
            if !current_chunk.is_empty() {
                chunks.push(current_chunk.clone());
                current_chunk.clear();
            }

            // If a single line is longer than max_size, split it
            if line.len() > max_size {
                let mut remaining = line;
                while remaining.len() > max_size {
                    let (chunk, rest) = remaining.split_at(max_size);
                    chunks.push(chunk.to_string());
                    remaining = rest;
                }
                if !remaining.is_empty() {
                    current_chunk.push_str(remaining);
                    current_chunk.push('\n');
                }
            } else {
                current_chunk.push_str(line);
                current_chunk.push('\n');
            }
        } else {
            current_chunk.push_str(line);
            current_chunk.push('\n');
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

/// Filter out compilation and build messages from cargo output
fn filter_compilation_output(stderr: &str) -> String {
    let mut filtered_lines = Vec::new();
    
    for line in stderr.lines() {
        // Skip compilation-related messages
        if line.trim().starts_with("Compiling ")
            || line.trim().starts_with("Finished ")
            || line.trim().starts_with("Running ")
            || line.trim().starts_with("Building ")
            || line.contains("target(s) in")
            || line.contains("Blocking waiting for file lock")
            || line.contains("Checking ")
            || line.trim().is_empty()
        {
            continue;
        }
        
        // Keep actual error/warning messages
        filtered_lines.push(line);
    }
    
    filtered_lines.join("\n")
}

/// Parse swap output and create a beautiful Discord embed
fn parse_swap_output_to_embed(output: &str) -> Option<CreateEmbed> {
    // Check if this is a swap test result
    if !output.contains("═══") && !output.contains("Swap Test Result") && !output.contains("Final Run Summary") {
        return None;
    }

    let mut embed = CreateEmbed::default();
    
    // Parse the output for key information
    let mut status = "Unknown";
    let mut pair = String::new();
    let mut order_id = String::new();
    let mut duration = String::new();
    let mut deposit_address = String::new();
    let mut src_init_tx = String::new();
    let mut dst_redeem_tx = String::new();
    let mut error_msg = String::new();
    let mut is_batch = false;
    let mut total_swaps = 0;
    let mut completed = 0;
    let mut failed = 0;
    let mut timed_out = 0;
    let mut run_id = String::new();
    
    // For batch results, collect individual swap details
    let mut batch_details: Vec<(String, String, String, String)> = Vec::new(); // (emoji, pair, order_id, error)

    for line in output.lines() {
        let line = line.trim();
        
        // Check for batch summary
        if line.contains("Final Run Summary") {
            is_batch = true;
        }
        
        // Parse Run ID
        if line.starts_with("Run ID") || line.starts_with("Run ID:") {
            run_id = line.split(':').skip(1).collect::<Vec<_>>().join(":").trim().to_string();
        }
        
        if line.starts_with("Pair") || line.starts_with("Pair:") {
            pair = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.starts_with("Status") || line.starts_with("Status:") {
            status = line.split(':').nth(1).unwrap_or("Unknown").trim();
        } else if line.starts_with("Order ID") || line.starts_with("Order ID:") {
            order_id = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.starts_with("Duration") || line.starts_with("Duration:") {
            duration = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.starts_with("Deposit") || line.starts_with("Deposit:") {
            deposit_address = line.split(':').skip(1).collect::<Vec<_>>().join(":").trim().to_string();
        } else if line.starts_with("Src Init") || line.starts_with("Src Init:") {
            src_init_tx = line.split(':').skip(1).collect::<Vec<_>>().join(":").trim().to_string();
        } else if line.starts_with("Dst Redeem") || line.starts_with("Dst Redeem:") {
            dst_redeem_tx = line.split(':').skip(1).collect::<Vec<_>>().join(":").trim().to_string();
        } else if line.starts_with("Error") || line.starts_with("Error:") {
            error_msg = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.starts_with("Total") || line.starts_with("Total:") {
            if let Ok(num) = line.split(':').nth(1).unwrap_or("0").trim().parse::<i32>() {
                total_swaps = num;
            }
        } else if line.starts_with("Completed") || line.starts_with("Completed:") {
            if let Ok(num) = line.split(':').nth(1).unwrap_or("0").trim().parse::<i32>() {
                completed = num;
            }
        } else if line.starts_with("Failed") || line.starts_with("Failed:") {
            if let Ok(num) = line.split(':').nth(1).unwrap_or("0").trim().parse::<i32>() {
                failed = num;
            }
        } else if line.starts_with("Timed Out") || line.starts_with("Timed Out:") {
            if let Ok(num) = line.split(':').nth(1).unwrap_or("0").trim().parse::<i32>() {
                timed_out = num;
            }
        } else if line.starts_with("Pending") || line.starts_with("Pending:") {
            if let Ok(num) = line.split(':').nth(1).unwrap_or("0").trim().parse::<i32>() {
                // Pending swaps are still processing
                if num > 0 {
                    // Add to description or handle separately
                }
            }
        }
        
        // Parse individual swap lines in batch output (format: "✅ pair | status | error")
        // Also handle "⏳" for pending/processing swaps
        if is_batch && (line.starts_with("✅") || line.starts_with("❌") || line.starts_with("⏰") || line.starts_with("↩️") || line.starts_with("⏳")) {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let emoji = parts[0];
                let rest = parts[1];
                let details: Vec<&str> = rest.split('|').collect();
                if details.len() >= 2 {
                    let swap_pair = details[0].trim().to_string();
                    let _swap_status = details[1].trim().to_string(); // Status already indicated by emoji
                    let swap_error = if details.len() > 2 { details[2].trim().to_string() } else { String::new() };
                    
                    // Try to extract order ID from the pair or status (if available in output)
                    // For now, we'll leave it empty as the CLI output doesn't include it in the summary
                    batch_details.push((emoji.to_string(), swap_pair, String::new(), swap_error));
                }
            }
        }
    }

    // Set color based on status
    let color = match status {
        "Completed" => Colour::from_rgb(0, 255, 0), // Green
        "Failed" => Colour::from_rgb(255, 0, 0),    // Red
        "TimedOut" => Colour::from_rgb(255, 165, 0), // Orange
        _ => Colour::from_rgb(100, 100, 100),       // Gray
    };

    embed = embed.colour(color);

    if is_batch {
        // Batch summary embed
        let mut description = format!(
            "**Total Swaps:** {}\n**✅ Completed:** {}\n**❌ Failed:** {}\n**⏰ Timed Out:** {}\n\n",
            total_swaps, completed, failed, timed_out
        );
        
        if !run_id.is_empty() {
            description.push_str(&format!("**Run ID:** `{}`\n\n", run_id));
        }
        
        // Add individual swap details
        if !batch_details.is_empty() {
            description.push_str("**Swap Details:**\n");
            for (emoji, swap_pair, _order_id, error) in batch_details.iter().take(15) { // Limit to 15 to avoid hitting Discord limits
                // Format the swap pair for better readability
                let formatted_pair = if swap_pair.contains("->") {
                    let parts: Vec<&str> = swap_pair.split("->").collect();
                    if parts.len() == 2 {
                        format!("{} → {}", parts[0].trim(), parts[1].trim())
                    } else {
                        swap_pair.clone()
                    }
                } else {
                    swap_pair.clone()
                };
                
                description.push_str(&format!("{} {}\n", emoji, formatted_pair));
                if !error.is_empty() {
                    description.push_str(&format!("   └─ `{}`\n", error));
                }
            }
            
            if batch_details.len() > 15 {
                description.push_str(&format!("\n_...and {} more swaps_\n", batch_details.len() - 15));
            }
        }
        
        embed = embed
            .title("🔄 Batch Swap Test Results")
            .description(description);
        
        if failed > 0 || timed_out > 0 {
            embed = embed.footer(CreateEmbedFooter::new("⚠️ Some swaps failed or timed out. Check full output for details."));
        } else {
            embed = embed.footer(CreateEmbedFooter::new("✅ All swaps completed successfully!"));
        }
    } else {
        // Single swap embed
        let status_emoji = match status {
            "Completed" => "✅",
            "Failed" => "❌",
            "TimedOut" => "⏰",
            "Refunded" => "↩️",
            _ => "⏳",
        };

        // Parse the pair to extract from and to assets
        let (from_asset, to_asset) = if pair.contains("->") {
            let parts: Vec<&str> = pair.split("->").collect();
            if parts.len() == 2 {
                (parts[0].trim().to_string(), parts[1].trim().to_string())
            } else {
                (pair.clone(), String::new())
            }
        } else {
            (pair.clone(), String::new())
        };

        // Create a more descriptive title
        let title = if !from_asset.is_empty() && !to_asset.is_empty() {
            format!("{} Cross-Chain Swap: {} → {}", status_emoji, from_asset, to_asset)
        } else {
            format!("{} Swap Test Result", status_emoji)
        };

        embed = embed.title(title);

        // Add From and To asset fields for better readability
        if !from_asset.is_empty() {
            embed = embed.field("From Asset", format!("`{}`", from_asset), true);
        }
        
        if !to_asset.is_empty() {
            embed = embed.field("To Asset", format!("`{}`", to_asset), true);
        }

        embed = embed.field("📊 Status", format!("{} {}", status_emoji, status), true);

        if !order_id.is_empty() && order_id != "N/A" {
            // Make Order ID easily copyable in a code block
            embed = embed.field("🔑 Order ID (click to copy)", format!("```\n{}\n```", order_id), false);
        }

        if !duration.is_empty() {
            embed = embed.field("⏱️ Duration", duration, true);
        }

        if !deposit_address.is_empty() {
            embed = embed.field("💰 Deposit Address", format!("`{}`", deposit_address), false);
        }

        if !src_init_tx.is_empty() {
            let short_tx = if src_init_tx.len() > 20 {
                format!("{}...{}", &src_init_tx[..10], &src_init_tx[src_init_tx.len()-10..])
            } else {
                src_init_tx.clone()
            };
            embed = embed.field("📤 Source Transaction", format!("`{}`", short_tx), false);
        }

        if !dst_redeem_tx.is_empty() {
            let short_tx = if dst_redeem_tx.len() > 20 {
                format!("{}...{}", &dst_redeem_tx[..10], &dst_redeem_tx[dst_redeem_tx.len()-10..])
            } else {
                dst_redeem_tx.clone()
            };
            embed = embed.field("📥 Destination Transaction", format!("`{}`", short_tx), false);
        }

        // Enhanced error handling with new notification format
        if !error_msg.is_empty() {
            // Categorize the error
            let (error_category, helpful_tip) = categorize_error(&error_msg);
            
            // Determine status type for notification format
            let status_type = if error_category == "Insufficient Balance" {
                "insufficient_funds"
            } else if error_category == "Liquidity Error" {
                "liquidity_error"
            } else if status == "Pending" {
                "in_processing"
            } else if status == "TimedOut" {
                "timeout"
            } else {
                "failed"
            };
            
            // Build warnings list
            let mut warnings = Vec::new();
            if !helpful_tip.is_empty() {
                warnings.push(helpful_tip.to_string());
            }
            
            // Use the new notification format for failed swaps
            if status == "Failed" || status == "TimedOut" {
                return Some(build_swap_notification_embed(
                    status_type,
                    &order_id,
                    &pair,
                    &error_msg,
                    warnings,
                ));
            }
            
            // Original format for other cases
            embed = embed.field(
                format!("⚠️ Error: {}", error_category),
                format!("```\n{}\n```", error_msg),
                false
            );
            
            if !helpful_tip.is_empty() {
                embed = embed.field("💡 Suggestion", helpful_tip, false);
            }
        }

        // Add footer based on status
        let footer_text = match status {
            "Completed" => "✅ Swap completed successfully!",
            "Failed" => "❌ Swap failed - check error details above",
            "TimedOut" => "⏰ Swap timed out - may still complete on-chain",
            "Refunded" => "↩️ Swap was refunded",
            _ => "Garden Finance Swap Tester",
        };
        
        embed = embed.footer(CreateEmbedFooter::new(footer_text));
    }

    Some(embed)
}

/// Autocomplete function for asset selection
async fn autocomplete_asset<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> Vec<String> {
    let assets = get_supported_assets();

    // Filter assets based on partial input
    let partial_lower = partial.to_lowercase();
    assets
        .into_iter()
        .filter(|asset| asset.to_lowercase().contains(&partial_lower))
        .map(|s| s.to_string())
        .take(25) // Discord limits to 25 suggestions
        .collect()
}

/// Get list of all supported assets (centralized for consistency)
fn get_supported_assets() -> Vec<&'static str> {
    vec![
        // Bitcoin
        "bitcoin_testnet:btc",
        // Ethereum Sepolia
        "ethereum_sepolia:eth",
        "ethereum_sepolia:wbtc",
        "ethereum_sepolia:usdc",
        // Base Sepolia
        "base_sepolia:wbtc",
        "base_sepolia:usdc",
        // Arbitrum Sepolia
        "arbitrum_sepolia:wbtc",
        "arbitrum_sepolia:usdc",
        // Solana Testnet
        "solana_testnet:sol",
        "solana_testnet:usdc",
        // Starknet Sepolia
        "starknet_sepolia:wbtc",
        // Tron Shasta
        "tron_shasta:usdt",
        "tron_shasta:wbtc",
        // Alpen Testnet
        "alpen_testnet:sbtc",
        "alpen_testnet:usdc",
        // BNB Chain Testnet
        "bnb_testnet:wbtc",
        // Citrea Testnet
        "citrea_testnet:usdc",
        // Monad Testnet
        "monad_testnet:usdc",
        // XRPL Testnet
        "xrpl_testnet:xrp",
    ]
}

/// Check if a swap pair is valid
fn is_valid_swap_pair(from_asset: &str, to_asset: &str) -> bool {
    let supported_assets = get_supported_assets();

    // Check if both assets are in the supported list
    if !supported_assets.contains(&from_asset) || !supported_assets.contains(&to_asset) {
        return false;
    }

    // Extract chain names
    let from_chain = from_asset.split(':').next().unwrap_or("");
    let to_chain = to_asset.split(':').next().unwrap_or("");

    // Assets must be on different chains
    if from_chain == to_chain {
        return false;
    }

    // All checks passed
    true
}

/// Categorize error messages and provide helpful suggestions
fn categorize_error(error_msg: &str) -> (&str, &str) {
    let error_lower = error_msg.to_lowercase();
    
    // Check for common error patterns and return (category, helpful_tip)
    if error_lower.contains("insufficient") && (error_lower.contains("balance") || error_lower.contains("funds")) {
        return (
            "Insufficient Balance",
            "💰 Make sure your wallet has enough testnet tokens. You can get testnet tokens from faucets for the respective chain."
        );
    }
    
    if error_lower.contains("network") || error_lower.contains("connection") || error_lower.contains("timeout") {
        return (
            "Network Error",
            "🌐 Network connectivity issue detected. Check your internet connection or try again in a few moments."
        );
    }
    
    if error_lower.contains("rpc") || error_lower.contains("provider") {
        return (
            "RPC Error",
            "🔌 RPC provider issue. The blockchain node might be temporarily unavailable. Try again later."
        );
    }
    
    if error_lower.contains("gas") || error_lower.contains("fee") {
        return (
            "Gas/Fee Error",
            "⛽ Transaction fee issue. Ensure you have enough native tokens (ETH, SOL, etc.) to cover gas fees."
        );
    }
    
    if error_lower.contains("nonce") {
        return (
            "Nonce Error",
            "🔢 Transaction nonce conflict. Wait a moment and try again, or check for pending transactions."
        );
    }
    
    if error_lower.contains("signature") || error_lower.contains("sign") {
        return (
            "Signature Error",
            "✍️ Transaction signing failed. Check your private key configuration in the .env file."
        );
    }
    
    if error_lower.contains("invalid") && error_lower.contains("address") {
        return (
            "Invalid Address",
            "📍 The wallet address format is invalid. Verify the address configuration."
        );
    }
    
    if error_lower.contains("slippage") {
        return (
            "Slippage Error",
            "📉 Price slippage exceeded limits. The market price moved too much during the swap."
        );
    }
    
    if error_lower.contains("liquidity") {
        return (
            "Liquidity Error",
            "💧 Insufficient liquidity for this swap pair. Try a smaller amount or different pair."
        );
    }
    
    if error_lower.contains("timeout") || error_lower.contains("timed out") {
        return (
            "Timeout",
            "⏰ The operation took too long. The swap might still complete on-chain. Check the order status later."
        );
    }
    
    if error_lower.contains("unauthorized") || error_lower.contains("forbidden") {
        return (
            "Authorization Error",
            "🔒 Access denied. Check your API keys and wallet permissions."
        );
    }
    
    if error_lower.contains("rate limit") {
        return (
            "Rate Limit",
            "🚦 Too many requests. Wait a moment before trying again."
        );
    }
    
    // Default category for unknown errors
    (
        "Unknown Error",
        "🔍 An unexpected error occurred. Check the error message above for details."
    )
}

/// Build a swap notification embed based on status type
/// Matches the format from the JavaScript Discord bot example
fn build_swap_notification_embed(
    status_type: &str,
    order_id: &str,
    order_pair: &str,
    reason: &str,
    warnings: Vec<String>,
) -> CreateEmbed {
    let (title, color, emoji) = match status_type {
        "failed" => (
            "❌ Swap Report — Failed Trade",
            Colour::from_rgb(231, 76, 60), // Red
            "❌"
        ),
        "insufficient_funds" => (
            "💸 Swap Report — Insufficient Funds",
            Colour::from_rgb(230, 126, 34), // Orange
            "💸"
        ),
        "liquidity_error" => (
            "🌊 Swap Report — Liquidity Error",
            Colour::from_rgb(155, 89, 182), // Purple
            "🌊"
        ),
        "in_processing" | "pending" => (
            "⏳ Swap Report — In Processing",
            Colour::from_rgb(52, 152, 219), // Blue
            "⏳"
        ),
        "completed" => (
            "✅ Swap Report — Trade Completed",
            Colour::from_rgb(46, 204, 113), // Green
            "✅"
        ),
        "timeout" => (
            "⏰ Swap Report — Timeout",
            Colour::from_rgb(255, 165, 0), // Orange
            "⏰"
        ),
        _ => (
            "❌ Swap Report — Failed Trade",
            Colour::from_rgb(231, 76, 60), // Red
            "❌"
        ),
    };

    let label = match status_type {
        "failed" => "Failure trade",
        "insufficient_funds" => "Insufficient funds",
        "liquidity_error" => "Liquidity error",
        "in_processing" | "pending" => "Processing trade",
        "completed" => "Successful trade",
        "timeout" => "Timeout",
        _ => "Failed trade",
    };

    let mut embed = CreateEmbed::default()
        .title(title)
        .colour(color)
        .timestamp(serenity::all::Timestamp::now())
        .field(format!("{} {}", emoji, label), "\u{200B}", false) // Section header with zero-width space
        .field("• Order ID", format!("`{}`", order_id), false)
        .field("• Order Pair", format!("`{}`", order_pair), false)
        .field("• Reason", reason, false);

    // Add warnings if present
    if !warnings.is_empty() {
        let warnings_text = warnings
            .iter()
            .map(|w| format!("• {}", w))
            .collect::<Vec<_>>()
            .join("\n");
        embed = embed.field("⚠️ Warnings", warnings_text, false);
    }

    embed
}

