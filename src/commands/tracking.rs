use std::time::Duration;
use poise::FrameworkError;
use poise::serenity_prelude as serenity;
use crate::Context;
use crate::db::{self, ActionGuard};

#[poise::command(
    slash_command,
    nsfw_only,
    check = "db::not_doing_something",
)]
/// Starts a stopwatch to time your premature ejaculation attempts.
pub async fn prejac(ctx: Context<'_>) -> anyhow::Result<()> {
    let _guard = ActionGuard::new(&ctx).await?;
    let start_time = crate::now();

    let reply = {
        let components = vec![serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new("I came!")
                .label("I came!")
                .emoji('💦')
                .style(serenity::ButtonStyle::Success),
            
            serenity::CreateButton::new("I give up...")
                .label("I give up!")
                .style(serenity::ButtonStyle::Danger),
        ])];

        poise::CreateReply::default()
            .ephemeral(true)
            .components(components)
            .content(format!("<@{}> started stroking <t:{}:R>!",
                ctx.author().id, start_time.as_secs()))
    };
    let reply = ctx.send(reply).await?;

    let interaction = serenity::ComponentInteractionCollector::new(ctx)
        .timeout(Duration::from_secs(60 * 5))
        .await;

    reply.delete(ctx).await?;
    if interaction.is_some_and(|i| i.data.custom_id == "I came!") {
        let end_time = crate::now();
        let pretty = crate::duration_string((end_time - start_time).as_secs());

        let msg = serenity::CreateMessage::default()
            .content(format!("<@{}> came in {}. Good gooner!",
                ctx.author().id, pretty));

        send_message(&ctx, msg).await?;
        db::log_prejac(&ctx, start_time.as_secs(), end_time.as_secs()).await?;
    }

    Ok(())
}

#[poise::command(
    slash_command,
    nsfw_only,
    check = "db::not_doing_something",
)]
/// Starts a stopwatch to time your gooning session.
pub async fn goon(ctx: Context<'_>) -> anyhow::Result<()> {
    db::start_gooning(&ctx).await?;

    let reply = poise::CreateReply::default()
        .ephemeral(true)
        .content("Great! 💦 I'll check in on you every 30 minutes, alright?\n\
            When you're done, send **/cum**");

    ctx.send(reply).await?;

    Ok(())
}

#[poise::command(
    slash_command,
    nsfw_only,
    check = "db::doing_something",
    on_error = "tried_cum",
)]
/// Wrap up your gooning session
pub async fn cum(ctx: Context<'_>) -> anyhow::Result<()> {
    let duration = db::stop_gooning(&ctx).await?;
    let pretty = crate::duration_string(duration);

    let msg = serenity::CreateMessage::default()
        .content(format!("<@{}> gooned for {}. Good gooner!",
            ctx.author().id, pretty));

    let reply = poise::CreateReply::default()
        .ephemeral(true)
        .content("Good gooner!");

    ctx.send(reply).await?;
    send_message(&ctx, msg).await?;

    Ok(())
}

#[poise::command(
    slash_command,
    nsfw_only,
)]
/// Check out your goon stats™
pub async fn stats(ctx: Context<'_>) -> anyhow::Result<()> {
    let colour = ctx.data().accent_colour;

    let best_times = db::best_times(&ctx).await?;
    let recent_times = db::recent_times(&ctx).await?;

    let best_gooning_times = format_best_times(&best_times.gooning);
    let best_prejac_times  = format_best_times(&best_times.prejac);
    let recent_gooning_times = format_recent_times(&recent_times.gooning);
    let recent_prejac_times  = format_recent_times(&recent_times.prejac);

    let msg = {
        let embeds = vec![
            serenity::CreateEmbed::default()
                .colour(colour)
                .title("Best Times")
                .field("😩 **Gooning**", best_gooning_times, true)
                .field("💦 **Prejac**",  best_prejac_times, true),
            serenity::CreateEmbed::default()
                .colour(colour)
                .title("Recent Times")
                .field("😩 **Gooning**", recent_gooning_times, true)
                .field("💦 **Prejac**",  recent_prejac_times, true),
        ];

        serenity::CreateMessage::default()
            .embeds(embeds)
            .content(format!("<@{}>'s gooning statistics",
                ctx.author().id.get()))
    };

    let reply = poise::CreateReply::default()
        .ephemeral(true)
        .content("Sent!");

    ctx.send(reply).await?;
    send_message(&ctx, msg).await?;

    Ok(())
}

fn format_best_times(times: &[(u64, u64)]) -> String {
    let r = ['🥇', '🥈', '🥉']
        .iter()
        .zip(times)
        .map(|(emoji, (end, start))|
            format!("{emoji} **{}**", crate::duration_string(end - start)))
        .collect::<Vec<String>>()
        .join("\n");
    
    if r.is_empty() {
        String::from("None!")
    } else {
        r
    }
}

fn format_recent_times(times: &[(u64, u64)]) -> String {
    let formatter = timeago::Formatter::new();
    let ts_now = crate::now().as_secs();

    let r = times
        .iter()
        .map(|(end, start)|
            format!("**{}** ({})",
                crate::duration_string(end - start),
                formatter.convert(Duration::from_secs(ts_now - *end))))
        .collect::<Vec<String>>()
        .join("\n");

    if r.is_empty() {
        String::from("None!")
    } else {
        r
    }
}

async fn tried_cum(err: FrameworkError<'_, crate::Data, anyhow::Error>) {
    if let FrameworkError::CommandCheckFailed { ctx, .. } = err {
        let reply = poise::CreateReply::default()
            .ephemeral(true)
            .content("You didn't tell me you were gooning!");
        
        let _ = ctx.send(reply).await;
    } else {
        crate::error_handler(err).await;
    }
}

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