use anyhow::Context as _;
use poise::serenity_prelude::{ClientBuilder, GatewayIntents, GuildId};
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let discord_guild_id = secret_store
        .get("DISCORD_GUILD_ID")
        .context("'DISCORD_GUILD_ID' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                hello(),
                parent(),               
                ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(ctx, &framework.options().commands, GuildId::new(discord_guild_id.parse::<u64>()
                .context("Failed to parse 'DISCORD_GUILD_ID'")?)).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = ClientBuilder::new(discord_token, GatewayIntents::non_privileged())
        .framework(framework)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}


/// Responds with "world!"
#[poise::command(slash_command)]
async fn hello(ctx: Context<'_>, args: String) -> Result<(), Error> {
    ctx.say("world!").await?;
    Ok(())
}

/// Parent?? :)
#[poise::command(slash_command, subcommands("child1", "child2"))]
pub async fn parent(ctx: Context<'_>, arg: String) -> Result<(), Error> {
    ctx.say(format!("Parent command received argument: {}", arg)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn child1(ctx: Context<'_>, arg: String) -> Result<(), Error> {
    ctx.say(format!("Child1 command received argument: {}", arg)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn child2(ctx: Context<'_>, arg: String) -> Result<(), Error> {
    ctx.say(format!("Child2 command received argument: {}", arg)).await?;
    Ok(())
}