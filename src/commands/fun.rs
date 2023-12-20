use std::time::Duration;
use poise::serenity_prelude as serenity;
use crate::{
    db::{self, set_prompt_frequency},
    edit_distance::{self, edit_distance},
    Context,
};

/// How often rk9 should check to see if a prompt has been written, in seconds.
const PROMPT_CHECK_INTERVAL: u64 = 2;
/// How long rk9 should be willing to wait for a prompt to have been written
/// in seconds.
const PROMPT_TIME_LIMIT: u64 = 60;

#[poise::command(slash_command, nsfw_only)]
/// Set how often you'd like to receive prompts.
pub async fn prompt(
    ctx: Context<'_>,
    #[description = "How many hours between prompts?"]
        frequency: Option<u64>,
) -> anyhow::Result<()>
{
    set_prompt_frequency(&ctx, frequency).await?;

    let reply = poise::CreateReply::default()
        .ephemeral(true)
        .content("Gotcha!");

    ctx.send(reply).await?;

    Ok(())
}

#[poise::command(slash_command, nsfw_only)]
/// Repeat after me!
pub async fn prompt_me(
    ctx: Context<'_>,
) -> anyhow::Result<()>
{
    let prompt = String::from("Furry porn has ruined me forever NGGGGGHHHH");
    let time_limit = crate::now() + Duration::from_secs(PROMPT_TIME_LIMIT);

    let reply = poise::CreateReply::default()
        .ephemeral(true)
        .content(format!("Repeat after me: {prompt}\nFails in <t:{}:R>",
            time_limit.as_secs()));
    let reply = ctx.send(reply).await?;

    let _ = tokio::time::timeout(time_limit, async move {
        let mut interval = tokio::time::interval(
            Duration::from_secs(PROMPT_CHECK_INTERVAL));

        loop {
            interval.tick().await;

            if let Some((msg, edit_distance::Similarity::Similar(dist))) =
                ctx.channel_id().messages(ctx.http(),
                    serenity::GetMessages::default().after(ctx.id())).await?
                .iter()
                .filter(|m| m.author.id == ctx.author().id)
                .map(   |m| (m, edit_distance(&prompt, &m.content)))
                .find(  |(_, dist)| *dist != edit_distance::Similarity::Dissimilar)
            {
                msg.react(ctx.http(), '🖤').await?;
                db::log_prompt_response(&ctx, dist).await?;
                return anyhow::Ok(())
            }
        }
    }).await;

    reply.delete(ctx).await?;

    Ok(())
}

