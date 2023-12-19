use poise::serenity_prelude as serenity;
use crate::{db, Context};

#[poise::command(
    slash_command,
    nsfw_only,
    guild_only,
    default_member_permissions = "ADMINISTRATOR",
)]
/// Limit where Rk9 will send messages.
/// Call with no argument to let Rk9 send messages anywhere.
pub async fn channel(ctx: Context<'_>, channel: Option<serenity::Channel>)
    -> anyhow::Result<()>
{
    // SAFETY: this function is guild_only, so it cannot be called from a DM.
    let guild = ctx.guild_id().unwrap().get();
    let channel = channel.map(|x| x.id().get());
    db::set_post_channel(&ctx, guild, channel).await?;

    let reply = poise::CreateReply::default()
        .ephemeral(true)
        .content("Done.");

    ctx.send(reply).await?;

    Ok(())
}