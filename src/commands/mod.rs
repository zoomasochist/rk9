use std::time::Duration;
use poise::serenity_prelude as serenity;
use crate::{db::{self, CumType}, Context};

pub mod tracking;
pub mod configure;
pub mod fun;

// Max time in seconds sessions are considered "recent"
const RECENT_TIMES_CUTOFF: u64 = 604_800;

#[poise::command(prefix_command, track_edits, slash_command)]
/// Shows a help menu
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> anyhow::Result<()>
{
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "\
        Type /help <command> for more info on a command.\n\
        Generally, using a feature command without an argument will turn it \
        off.",
        ..Default::default()
    };

    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

/// Send a message to either the channel of the context message, or the channel
/// specified by the server's channel setting.
async fn send_message(ctx: &Context<'_>, msg: serenity::CreateMessage)
    -> anyhow::Result<()>
{
    let channel = if let Some(guild_id) = ctx.guild_id() {
        db::post_channel(ctx, guild_id.get()).await?
            .unwrap_or_else(|| ctx.channel_id().get())
    } else {
        ctx.channel_id().get()
    };

    serenity::ChannelId::from(channel).send_message(ctx.http(), msg).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    nsfw_only,
)]
/// Check out your (or somebody elses) goon stats™
pub async fn stats(
    ctx: Context<'_>,
    user: Option<serenity::User>,
) -> anyhow::Result<()>
{
    let user_id = user.map_or(ctx.author().id.get(), |v| v.id.get());
    let gooning_times = db::gooning_times(&ctx, user_id).await?;
    let edging_times  = db::edging_times(&ctx, user_id).await?;

    let msg = serenity::CreateMessage::default()
        .add_embed(best_times_embed(&ctx, &gooning_times, &edging_times))
        .add_embed(recent_times_embed(&ctx, &gooning_times, &edging_times))
        .content(format!("<@{user_id}>'s gooning statistics!"));

    let reply = poise::CreateReply::default()
        .ephemeral(true)
        .content("Sent!");

    ctx.send(reply).await?;
    send_message(&ctx, msg).await?;

    Ok(())
}

fn best_times_embed(
    ctx: &Context<'_>,
    gooning: &[db::CumTime],
    edging:  &[db::CumTime],
) -> serenity::CreateEmbed
{
    let now = crate::now();
    let lang = timeago::Formatter::new();
    let colour = ctx.data().accent_colour;
    let user_id = ctx.author().id.get();

    let time_spent = gooning
        .iter()
        .chain(edging)
        .map(|m| m.ended_at - m.started_at)
        .sum();

    /* I don't want to destroy a passed vec :( */
    // let mut gooning = gooning.to_vec();
    // let mut edging  = edging.to_vec();

    // // Sort by "best" (longest for gooning, shortest for edging)
    // gooning.sort_by(|a, b| (a.ended_at - a.started_at).cmp(&(b.ended_at - b.started_at)));
    // edging .sort_by(|a, b| (b.ended_at - b.started_at).cmp(&(a.ended_at - a.started_at)));

    let fields = gooning
        .iter()
        .take(3)
        .chain(edging.iter().take(3))
        .map(|time| {
            let dur = time.ended_at - time.started_at;
            let time_name = crate::adj_duration_string(dur);

            let to = if time.description.is_empty() {
                String::new()
            } else {
                format!(" to **{}**", time.description.replace('*', ""))
            };

            let rel_t = now - Duration::from_secs(time.ended_at);
            let description = format!("{}{}",
                lang.convert(rel_t), to);

            let title = match time.typ {
                CumType::Gooning => format!("**😩 {time_name} gooning session**"),
                CumType::Prejac  => format!("**💦 {time_name} prejac**"),
            };
            
            (title, description, true)
        })
        .collect::<Vec<(String, String, bool)>>();

    serenity::CreateEmbed::default()
        .colour(colour)
        .fields(fields)
        .author(serenity::CreateEmbedAuthor::new(&ctx.author().name)
            .icon_url(ctx.author().avatar_url().unwrap_or_default()))
        .title("Best Times")
        .description(format!("<@{user_id}> has spent **{}** with porn in total!",
            crate::duration_string(time_spent)))
}

fn recent_times_embed(
    ctx: &Context<'_>,
    gooning: &[db::CumTime],
    edging:  &[db::CumTime],
) -> serenity::CreateEmbed
{
    let now = crate::now();
    let lang = timeago::Formatter::new();
    let colour = ctx.data().accent_colour;
    let user_id = ctx.author().id.get();

    let gooning = gooning
        .iter()
        .filter(|v| (now.as_secs() - v.ended_at) < RECENT_TIMES_CUTOFF)
        .collect::<Vec<&db::CumTime>>();
    let edging = edging
        .iter()
        .filter(|v| (now.as_secs() - v.ended_at) < RECENT_TIMES_CUTOFF)
        .collect::<Vec<&db::CumTime>>();

    let time_spent = gooning
        .iter()
        .chain(&edging)
        .map(|m| m.ended_at - m.started_at)
        .sum();

    let fields = gooning
        .iter()
        .take(3)
        .chain(edging.iter().take(3))
        .map(|time| {
            let dur = time.ended_at - time.started_at;
            let time_name = crate::adj_duration_string(dur);

            let to = if time.description.is_empty() {
                String::new()
            } else {
                format!(" to **{}**", time.description.replace('*', ""))
            };

            let rel_t = now - Duration::from_secs(time.ended_at);
            let description = format!("{}{}",
                lang.convert(rel_t), to);

            let title = match time.typ {
                CumType::Gooning => format!("**😩 {time_name} gooning session**"),
                CumType::Prejac  => format!("**💦 {time_name} prejac**"),
            };
            
            (title, description, true)
        })
        .collect::<Vec<(String, String, bool)>>();

    serenity::CreateEmbed::default()
        .colour(colour)
        .fields(fields)
        .author(serenity::CreateEmbedAuthor::new(&ctx.author().name)
            .icon_url(ctx.author().avatar_url().unwrap_or_default()))
        .title("Recent Times")
        .description(format!("<@{user_id}> has spent **{}** with porn this week!",
            crate::duration_string(time_spent)))
}