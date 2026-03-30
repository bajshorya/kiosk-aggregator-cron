use anyhow::Result;
use poise::serenity_prelude as serenity;
use tracing::{error, info};

use super::commands;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

// User data shared across all commands
pub struct Data {}

/// Start the Discord bot with slash commands
pub async fn start_discord_bot(token: String) -> Result<()> {
    info!("Starting Discord bot...");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::ping(),
                commands::test_swap(),
                commands::test_swap_all(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                info!("Bot is ready! Registering slash commands globally...");
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                info!("Slash commands registered successfully");
                Ok(Data {})
            })
        })
        .build();

    let intents = serenity::GatewayIntents::non_privileged();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    match client {
        Ok(mut client) => {
            info!("Discord client created, starting event loop...");
            if let Err(e) = client.start().await {
                error!("Discord client error: {}", e);
                return Err(e.into());
            }
        }
        Err(e) => {
            error!("Failed to create Discord client: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
