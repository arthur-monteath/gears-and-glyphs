mod character;

use anyhow::Context as _;
use poise::serenity_prelude as serenity;
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;
use std::{collections::HashMap, env::var, time::Duration};
use tokio::sync::Mutex;
use std::sync::Arc;

struct Character {
    name: String,
    class: CharacterClass,
    exp: u32,
}

#[derive(Debug, poise::ChoiceParameter)]
enum CharacterClass {
    #[name = "Warrior"]
    Warrior,
    #[name = "Mage"]
    Mage,
    #[name = "Engineer"]
    Engineer,
}

// User data, which is stored and accessible in all command invocations
struct Data {
    user_characters: Arc<Mutex<HashMap<String, Character>>>,
}

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
                character(),               
                ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(ctx, &framework.options().commands, serenity::GuildId::new(discord_guild_id.parse::<u64>()
                .context("Failed to parse 'DISCORD_GUILD_ID'")?)).await?;
                Ok(Data {
                    user_characters: Arc::new(Mutex::new(HashMap::new())),
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(discord_token, serenity::GatewayIntents::non_privileged())
        .framework(framework)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}

/// Commands related to Characters
#[poise::command(slash_command, subcommands("create", "modify", "select", "delete"))]
pub async fn character(ctx: Context<'_>, arg: String) -> Result<(), Error> {
    ctx.say(format!("Parent command received argument: {}", arg)).await?;
    Ok(())
}

/// Creates a new Character
#[poise::command(slash_command)]
pub async fn create(ctx: Context<'_>, 
    #[description = "The name of the Character"] name: String,
    #[description = "The class of the Character"] class: CharacterClass
    ) -> Result<(), Error> {
    
    let mut user_characters = ctx.data().user_characters.lock().await;

    let character = Character {
        name: name.clone(),
        class: class,
        exp: 0,
    };

    ctx.say(format!("Character named {} has been Created.", character.name.clone())).await?;

    user_characters.insert(character.name.clone(), character);

    Ok(())
}

/// Modifies an existing Character
#[poise::command(slash_command)]
pub async fn modify(ctx: Context<'_>) -> Result<(), Error> {
    // TODO
    Ok(())
}

/// Selects a Character
#[poise::command(slash_command)]
pub async fn select(ctx: Context<'_>) -> Result<(), Error> {
    // TODO
    Ok(())
}

/// Deletes a Character
#[poise::command(slash_command)]
pub async fn delete(ctx: Context<'_>) -> Result<(), Error> {
    // TODO
    Ok(())
}