#![warn(clippy::pedantic)]
#![feature(let_chains)]
use tokio::sync::Mutex;
use poise::{serenity_prelude as serenity, FrameworkError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use log::info;

mod config;
mod db;
mod commands;
mod edit_distance;

pub(crate) type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

struct Data {
    accent_colour: u32,
    strings: Strings,
    db: Mutex<rusqlite::Connection>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Strings {
    stupid_things: Vec<String>,
    mean: Vec<String>,
    extreme: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config: config::Config
        = toml::from_str(&std::fs::read_to_string("./rk9.toml")?)?;

    let commands: Vec<poise::Command<Data, anyhow::Error>> = vec![
        // Misc
        commands::help(),
        commands::stats(),
        // Admin
        commands::configure::channel(),
        // Tracking
        commands::tracking::prejac(),
        commands::tracking::goon(),
        commands::tracking::cum(),
        // Fun
        commands::fun::prompt(),
        commands::fun::prompt_me(),
    ];

    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            on_error: |e| { Box::pin(error_handler(e)) },
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| Box::pin(async move {
            info!("Logged in as {}", ready.user.name);

            let strings =
                toml::from_str(&std::fs::read_to_string("strings.toml")?)?;

            let db = rusqlite::Connection::open(config.database_path)?;
            db::migrations(&db)?;

            poise::builtins::register_globally(
                ctx,
                &framework.options().commands).await?;

            info!("Ready");
            Ok(Data {
                accent_colour: config.accent_colour,
                strings,
                db: db.into()
            })
        }))
        .build();
    
    let client = serenity::ClientBuilder::new(config.discord_token, intents)
        .framework(framework)
        .await;

    client?.start().await?;
    Ok(())
}

async fn error_handler(err: FrameworkError<'_, Data, anyhow::Error>) {
    match err {
        FrameworkError::CommandCheckFailed { ctx, .. } => {
            let reply = poise::CreateReply::default()
                .ephemeral(true)
                .content("Looks like you're already up to something!");

            let _ = ctx.send(reply).await;
        },
        FrameworkError::NsfwOnly { ctx, .. } => {
            let reply = poise::CreateReply::default()
                .ephemeral(true)
                .content("Sorry, this isn't a NSFW channel!");
            
            let _ = ctx.send(reply).await;
        },
        FrameworkError::GuildOnly { ctx, .. } => {
            let reply = poise::CreateReply::default()
                .ephemeral(true)
                .content("You can only use this command from within a server.");
            
            let _ = ctx.send(reply).await;
        },
        FrameworkError::Command { error, ctx, .. } => {
            let reply = poise::CreateReply::default()
                .ephemeral(true)
                .content("Internal error! Please report this.");
        
            let _ = ctx.send(reply).await;
            log::error!("{}", error);
        },
        FrameworkError::Setup { error, .. } => {
            log::error!("Couldn't initialise bot! {}", error);
            std::process::exit(1);
        },
        _ => log::error!("Unmatched error"),
    }
}

/// Convenience function to return the current Unix timestamp in seconds.
fn now() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

/// Convert a number of seconds to a string of the format
/// h hours, m minutes, s seconds
fn duration_string(dur: u64) -> String {
    let seconds = dur % 60;
    let minutes = (dur / 60) % 60;
    let hours   = (dur / 60) / 60;

    // Is this stupid? I dunno
    match (seconds, minutes, hours) {
        (0, 0, 0) => String::from("zero"),
        (s, 0, 0) => format!("{s} seconds"),
        (0, m, 0) => format!("{m} minutes"),
        (0, 0, h) => format!("{h} hours"),
        (0, m, h) => format!("{h} hours, {m} minutes"),
        (s, 0, h) => format!("{h} hours, {s} seconds"),
        (s, m, 0) => format!("{m} minutes, {s} seconds"),
        (s, m, h) => format!("{h} hours, {m} minutes, {s} seconds"),
    }
}

fn adj_duration_string(dur: u64) -> String {
    let seconds = dur % 60;
    let minutes = (dur / 60) % 60;
    let hours   = (dur / 60) / 60;

    match (seconds, minutes, hours) {
        (s, 0, 0) => format!("{s} second"),
        (_, m, 0) => format!("{m} minute"),
        (_, 0, h) => format!("{h} hour"),
        (_, m, h) => format!("{h} hour, {m} minute"),
    }
}