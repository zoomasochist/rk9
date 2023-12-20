use std::time::Duration;
use poise::{Modal, FrameworkError};
use poise::serenity_prelude as serenity;
use crate::{db::{self, ActionGuard}, Context, commands::send_message};

#[derive(Default, Modal)]
#[name = "Cum Information Form 1040"]
struct CumModal {
    #[name = "What'd you cum to?"]
    #[placeholder = "List some tags, fetishes, artists.. my mom?"]
    #[max_length = 128]
    #[paragraph]
    description: Option<String>,
}

type ApplicationContext<'a> = poise::ApplicationContext<'a, crate::Data, anyhow::Error>;

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
    
    if let Some(int) = interaction && int.data.custom_id == "I came!" {
        let end_time = crate::now();
        let pretty = crate::duration_string((end_time - start_time).as_secs());

        let resp = poise::modal::execute_modal_on_component_interaction(
            &ctx, int, Some(CumModal::default()), None,
        ).await?.unwrap_or_default();
        let came_to = resp.description.unwrap_or_default();

        // This I cannot call clever.
        let no_inp = came_to.is_empty();
        let came_to_ = format!("{}**{}**{}",
            if no_inp { "" } else { " to " },
            came_to,
            if no_inp { "." } else { "" });

        let msg = serenity::CreateMessage::default()
            .content(format!("<@{}> came in {}{}!",
                ctx.author().id, pretty, came_to_));

        send_message(&ctx, msg).await?;
        db::log_prejac(
            &ctx,
            start_time.as_secs(),
            end_time.as_secs(),
            &came_to,
        ).await?;        
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
/// Wrap up your gooning session.
pub async fn cum(ctx: ApplicationContext<'_>) -> anyhow::Result<()> {
    let gctx = poise::Context::Application(ctx);
    let now = crate::now().as_secs();
    
    let resp = CumModal::execute(ctx).await?.unwrap_or_default();
    let came_to = resp.description.unwrap_or_default();

    let duration = db::stop_gooning(&gctx, now, &came_to).await?;
    let pretty = crate::duration_string(duration);

    // This I cannot call clever.
    let no_inp = came_to.is_empty();
    let came_to = format!("{}**{}**{}",
        if no_inp { "" } else { " to " },
        came_to,
        if no_inp { "." } else { "" });

    let msg = serenity::CreateMessage::default()
        .content(format!("<@{}> gooned for {}{}!",
            ctx.author().id, pretty, came_to));

    let reply = poise::CreateReply::default()
        .ephemeral(true)
        .content("Good gooner!");

    ctx.send(reply).await?;
    send_message(&gctx, msg).await?;

    Ok(())
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
