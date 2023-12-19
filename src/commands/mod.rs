use crate::Context;

pub mod tracking;
pub mod configure;

#[poise::command(prefix_command, track_edits, slash_command)]
/// Shows a help menu
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> anyhow::Result<()>
{
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "Type /help <command> for more info on a command.",
        ..Default::default()
    };

    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}