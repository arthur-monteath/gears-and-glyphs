mod character;
mod duelingSystem;

use duelingSystem::Duel;

use anyhow::Context as _;
use poise::{reply, Modal, serenity_prelude as serenity};
use serenity::all::{CreateInteractionResponse, InteractionType};
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;
use std::{collections::HashMap, f32::consts::E, sync::Arc};
use tokio::sync::Mutex;

#[derive(Modal)]
struct ActionModal {
    #[name = "Tile Input"]
    tile: String,
}

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
    user_characters: Arc<Mutex<HashMap<serenity::UserId, HashMap<String, Character>>>>,
    duels: Arc<Mutex<HashMap<serenity::UserId, Duel>>>
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
                duel(),            
                ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(ctx, &framework.options().commands, serenity::GuildId::new(discord_guild_id.parse::<u64>()
                .context("Failed to parse 'DISCORD_GUILD_ID'")?)).await?;
                Ok(Data {
                    user_characters: Arc::new(Mutex::new(HashMap::new())),
                    duels: Arc::new(Mutex::new(HashMap::new())),
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

async fn autocomplete_character(
    ctx: Context<'_>,
    partial: &str,
) -> impl Iterator<Item = String> {
    let user_id = ctx.author().id;
    let user_characters = ctx.data().user_characters.lock().await;

    user_characters
        .get(&user_id)
        .map_or_else(Vec::new, |chars| {
            chars.keys()
                .filter(|name| name.to_lowercase().starts_with(&partial.to_lowercase()))
                .cloned()
                .collect()
        })
        .into_iter()
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

    let user_id = ctx.author().id;

    let user_entry = user_characters.entry(user_id).or_insert_with(HashMap::new);
    user_entry.insert(character.name.clone(), character);

    ctx.say(format!("Character named {} has been Created.", name)).await?;

    Ok(())
}

/// Modifies an existing Character
#[poise::command(slash_command)]
pub async fn modify(
    ctx: Context<'_>,
    #[description = "Select a character to modify"] #[autocomplete = "autocomplete_character"] character: String,
    #[description = "New name for the character"] new_name: Option<String>,
) -> Result<(), Error> {
    let user_id = ctx.author().id;
    let mut user_characters = ctx.data().user_characters.lock().await;

    if let Some(user_entry) = user_characters.get_mut(&user_id) {
        if let Some(character_selected) = user_entry.remove(&character) {
            
            let updated_character = Character {
                name: new_name.clone().unwrap_or(character_selected.name),
                class: character_selected.class,
                exp: character_selected.exp
            };
            
            let new_character_name = updated_character.name.clone();
            user_entry.insert(new_character_name.clone(), updated_character);

            ctx.say(format!("Character has been modified to {}.", new_character_name)).await?;
        } else {
            ctx.say("Character not found!").await?;
        }
    } else {
        ctx.say("You have no characters!").await?;
    }

    Ok(())
}

/// Selects a Character
#[poise::command(slash_command)]
pub async fn select(
    ctx: Context<'_>,
    #[description = "Select a character"] #[autocomplete = "autocomplete_character"] character: String,
) -> Result<(), Error> {
    let user_id = ctx.author().id;
    let user_characters = ctx.data().user_characters.lock().await;

    if let Some(user_entry) = user_characters.get(&user_id) {
        if let Some(character_selected) = user_entry.get(&character) {
            ctx.say(format!("You selected the character: {} who is a {:?}", character_selected.name, character_selected.class)).await?;
        } else {
            ctx.say("Character not found!").await?;
        }
    } else {
        ctx.say("You have no characters!").await?;
    }

    Ok(())
}

/// Deletes a Character
#[poise::command(slash_command)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "Select a character to delete"] #[autocomplete = "autocomplete_character"] character: String
) -> Result<(), Error> {
    let user_id = ctx.author().id;
    let mut user_characters = ctx.data().user_characters.lock().await;

    if let Some(user_entry) = user_characters.get_mut(&user_id) {
        if user_entry.remove(&character).is_some() {
            ctx.say(format!("Character {} has been deleted.", character)).await?;
        } else {
            ctx.say("Character not found!").await?;
        }
    } else {
        ctx.say("You have no characters!").await?;
    }

    Ok(())
}

/// Invites another User to a Duel
#[poise::command(slash_command)]
pub async fn duel(ctx: Context<'_>, 
    #[description = "User to invite to the duel"] invitee: serenity::User
    ) -> Result<(), Error> {
    
    let inviter_id = ctx.author().id;
    let invitee_id = invitee.id;

    // Build the interaction response with a button to accept the duel
    let content = format!("<@{}> has invited <@{}> to a duel!", ctx.author().id, invitee.id);

    let reply = ctx
        .send(poise::CreateReply::default().content(content)
        .components(vec![
            serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new("accept")
                    .label("Accept")
                    .style(serenity::ButtonStyle::Success)
                    .emoji('✔'),
                serenity::CreateButton::new("decline")
                    .label("Decline")
                    .style(serenity::ButtonStyle::Danger)
                    .emoji('✖'),
            ])
        ])
    ).await?;

    let interaction = reply
        .message()
        .await?
        .await_component_interaction(ctx)
        .author_id(invitee_id)
        .await;
    
    reply
        .edit(ctx, poise::CreateReply::default().content("Processing... Please wait.").components(vec![]))
        .await?; // remove buttons after button press and edit message
        
    let pressed_button_id = match &interaction {
        Some(m) => &m.data.custom_id,
        None => {
            ctx.say(":warning: You didn't interact in time - please run the command again.").await?;
            return Ok(());
        }
    };

    let decision = match &**pressed_button_id {
        "accept" => true,
        "decline" => false,
        other => {
            tracing::warn!("unknown register button ID: {:?}", other);
            return Ok(());
        }
    };

    if decision
    {
        let mut duels = ctx.data().duels.lock().await;

        let new_duel = Duel::new(inviter_id, invitee_id);

        duels.insert(inviter_id, new_duel);

        let board = duels.get(&inviter_id).unwrap().get_board();

        reply.edit(ctx, poise::CreateReply::default().content(board).components(vec![])).await?;

        // Start the turn loop
        let mut current_player = inviter_id;
        loop {
            if let Err(e) = send_turn_message(ctx, current_player).await {
                ctx.say(format!("Error: {}", e)).await?;
                break;
            }
            
            // Alternate turns
            current_player = if current_player == inviter_id { invitee_id } else { inviter_id };
        }

        return Ok(())
    }
    
    reply.edit(ctx, poise::CreateReply::default().content("Duel declined.").components(vec![])).await?;

    Ok(())
}

async fn send_turn_message(ctx: Context<'_>, player_id: serenity::UserId) -> Result<(String, String), Error> {
    let reply = ctx.send(poise::CreateReply::default().ephemeral(true).content("Your turn! Choose an action:")
        .components(vec![
            serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new("atk")
                    .label("Attack")
                    .style(serenity::ButtonStyle::Danger)
                    .emoji('⚔'),
                serenity::CreateButton::new("mov")
                    .label("Move")
                    .style(serenity::ButtonStyle::Success)
                    .emoji('🏃'),
                serenity::CreateButton::new("use")
                    .label("Use")
                    .style(serenity::ButtonStyle::Primary)
                    .emoji('✍'),
            ])
        ])
    ).await?;

    if let Some(interaction) = reply
        .message()
        .await?
        .await_component_interaction(ctx)
        .author_id(player_id)
        .await
        {
            let action = match interaction.data.custom_id.as_str() {
                "atk" => "Attack",
                "mov" => "Move",
                "use" => "Use",
                _ => return Ok(("Attack".to_string(), "A1".to_string())), // Unexpected button id
            };

            // Create and show the modal in response to the button interaction
            let modal = serenity::CreateInteractionResponse::Modal(serenity::CreateModal::new(
            "action_modal",
            format!("Where do you want {} to take place?", action))
            .components(vec![
                serenity::CreateActionRow::InputText(serenity::CreateInputText::new(
                    serenity::InputTextStyle::Short,
                    "Where?",
                    "tile_input")
                    .placeholder("A1")
                    .required(true)
                    )
                ])
            );

            // Respond with the modal
            interaction.create_response(&ctx.http(), modal).await?;

            // Await the modal submission using event system
            let interaction_token = interaction.token.clone();

            if let Some(modal_submission) = serenity::ModalInteractionCollector::new(ctx)
            .author_id(player_id)
            .timeout(std::time::Duration::from_secs(60))
            .filter(move |modal| modal.token == interaction_token)
            .await
            {
                // Extract the input value from the modal submission
                if let Some(action_input) = modal_submission.data.components.get(0)
                .and_then(|row| row.components.get(0))
                .and_then(|component| {
                if let serenity::ActionRowComponent::InputText(input) = component {
                    Some(&input.value)
                } else {
                    None
                }
            })
            {
                return Ok((action.to_string(),action_input.clone().unwrap_or("A1".to_string())));
            }
        }
    }

    return Ok(("Attack".to_string(), "A1".to_string()));
}