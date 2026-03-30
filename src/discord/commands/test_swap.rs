use poise::serenity_prelude::CreateAttachment;
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
/// Example: /test-swap chain:ethereum_sepolia:eth base_sepolia:wbtc
#[poise::command(slash_command, prefix_command, rename = "test-swap")]
pub async fn test_swap(
    ctx: Context<'_>,
    #[description = "From asset (e.g., ethereum_sepolia:eth)"] from_asset: String,
    #[description = "To asset (e.g., base_sepolia:wbtc)"] to_asset: String,
    #[description = "Optional custom amount in smallest units"] amount: Option<String>,
) -> Result<(), Error> {
    info!(
        "Received /test-swap command: {} -> {} (amount: {:?})",
        from_asset, to_asset, amount
    );

    // Defer the response immediately to prevent timeout
    ctx.defer().await?;

    // Get the project root directory (where Cargo.toml is located)
    let project_root = env::current_dir()?;

    // Build the command arguments
    let mut args = vec!["run", "--release", "--quiet", "--", "test-swap", &from_asset, &to_asset];
    if let Some(ref amt) = amount {
        args.push(amt);
    }

    info!("Executing: cargo {} in {:?}", args.join(" "), project_root);

    // Spawn the cargo command asynchronously
    let mut child = Command::new("cargo")
        .args(&args)
        .current_dir(&project_root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Capture stdout and stderr
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    let mut stdout_reader = tokio::io::BufReader::new(stdout);
    let mut stderr_reader = tokio::io::BufReader::new(stderr);

    let mut stdout_content = String::new();
    let mut stderr_content = String::new();

    // Read all output
    stdout_reader.read_to_string(&mut stdout_content).await?;
    stderr_reader.read_to_string(&mut stderr_content).await?;

    // Wait for the process to complete
    let status = child.wait().await?;

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

    let args = vec!["run", "--release", "--quiet", "--", "run-once"];

    info!("Executing: cargo {} in {:?}", args.join(" "), project_root);

    // Spawn the cargo command asynchronously
    let mut child = Command::new("cargo")
        .args(&args)
        .current_dir(&project_root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Capture stdout and stderr
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    let mut stdout_reader = tokio::io::BufReader::new(stdout);
    let mut stderr_reader = tokio::io::BufReader::new(stderr);

    let mut stdout_content = String::new();
    let mut stderr_content = String::new();

    // Read all output
    stdout_reader.read_to_string(&mut stdout_content).await?;
    stderr_reader.read_to_string(&mut stderr_content).await?;

    // Wait for the process to complete
    let status = child.wait().await?;

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

/// Helper function to send output to Discord, handling the 2000 character limit
async fn send_output(ctx: Context<'_>, output: &str) -> Result<(), Error> {
    if output.len() <= DISCORD_MESSAGE_LIMIT {
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
