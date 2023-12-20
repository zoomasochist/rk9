// To avoid deadlocks, there should preferably be _no_ lines of code between
// the acquisition of a database lock, SQL execution, and return. 
use log::info;
use anyhow::Result;
use rusqlite::{OptionalExtension, types::Null};
use crate::Context;

mod internal;
pub use internal::{CumType, CumTime};
mod action_guard;
pub use action_guard::ActionGuard;

const MIGRATIONS: [&str; 1] = [
    include_str!("../../migrations/base.sql"),
];

pub fn migrations(db: &rusqlite::Connection) -> anyhow::Result<()> {
    for (idx, migration) in MIGRATIONS.iter().enumerate() {
        info!("migration {}: {:?}", idx, db.execute_batch(migration)?);
    }

    Ok(())
}

/// Records a prejac in `cum_times`.
pub async fn log_prejac(
    ctx: &Context<'_>,
    start: u64,
    end: u64,
    description: &str,
) -> Result<()>
{
    let db = ctx.data().db.lock().await;
    db.execute("
        INSERT INTO cum_times
            (user_id, started_at, ended_at, is_complete, what, description)
        VALUES      (?1, ?2, ?3, ?4, ?5, ?6)",
        (ctx.author().id.get(), start, end, true, "prejac", description))?;
    
    Ok(())
}

/// Adds an entry for the user in `cum_times` and sets `doing_something`.
pub async fn start_gooning(
    ctx: &Context<'_>
) -> Result<()>
{
    internal::create_user(ctx).await?;

    let now = crate::now().as_secs();
    let author = ctx.author().id.get();

    let db = ctx.data().db.lock().await;
    db.execute("
        INSERT INTO cum_times
               (user_id, started_at, ended_at, is_complete, what)
        VALUES (?1, ?2, ?3, ?4, ?5)",
        (author, now, Null, false, "gooning"))?;
    db.execute("UPDATE users SET doing_something = 1 WHERE id = ?1",
        [author])?;

    Ok(())
}

/// Ties up the user's current session by setting the active `cum_times`
/// `ended_at` to the current time and resetting `doing_something`.
pub async fn stop_gooning(
    ctx: &Context<'_>,
    time: u64,
    description: &str,
) -> Result<u64>
{
    internal::create_user(ctx).await?;
    
    let author = ctx.author().id.get();

    let db = ctx.data().db.lock().await;
    let result: u64 = db.query_row_and_then("
        UPDATE    cum_times
        SET       ended_at    = ?1,
                  description = ?2,
                  is_complete = true
        WHERE     user_id     = ?3 AND is_complete = false AND what = 'gooning'
        RETURNING ended_at, started_at",
        (time, description, ctx.author().id.get()),
        |row| anyhow::Ok(u64::saturating_sub(row.get(0)?, row.get(1)?)))?;

    db.execute("UPDATE users SET doing_something = 0 WHERE id = ?1",
        [author])?;

    Ok(result)
}

/// Returns whether the user's `doing_something` field is set, i.e. the user
/// is currently using /gooning or /prejac.
pub async fn doing_something(ctx: Context<'_>) -> Result<bool> {
    internal::create_user(&ctx).await?;

    let db = ctx.data().db.lock().await;
    let r = db.query_row(
            "SELECT doing_something FROM users WHERE id = ?1",
            [ctx.author().id.get()],
            |row| row.get(0))?;

    Ok(r)
}

/// Inverse of `doing_something`.
/// Useful for command predicates where we cant just !
pub async fn not_doing_something(ctx: Context<'_>) -> Result<bool> {
    doing_something(ctx).await.map(|v| !v)
}

/// Sets the sole channel in which Rk9 can post. Call with `channel_id = None`
/// to disable.
pub async fn set_post_channel(
    ctx: &Context<'_>,
    guild_id: u64,
    channel_id: Option<u64>,
) -> Result<()>
{
    internal::create_server_configuration(ctx, guild_id).await?;

    let db = ctx.data().db.lock().await;
    db.execute("
        UPDATE server_configurations
        SET    post_channel = ?1
        WHERE  id = ?2",
        (channel_id, guild_id))?;

    Ok(())
}

/// Retrieve the `guild_id`'s post channel, if any.
pub async fn post_channel(
    ctx: &Context<'_>,
    guild_id: u64
) -> Result<Option<u64>>
{
    internal::create_server_configuration(ctx, guild_id).await?;

    let db = ctx.data().db.lock().await;

    let r: Option<u64> = db
        .query_row(
            "SELECT post_channel FROM server_configurations WHERE id = ?1",
            [guild_id],
            |row| row.get(0),
        )
        .optional()?
        .flatten();

    Ok(r)
}

pub async fn set_prompt_frequency(
    ctx: &Context<'_>,
    freq: Option<u64>
) -> Result<()>
{
    internal::create_user(ctx).await?;
    let user = ctx.author().id.get();

    let db = ctx.data().db.lock().await;
    db.execute("
        UPDATE users
        SET    prompt_frequency = ?1
        WHERE  id = ?2
    ", (freq, user))?;

    Ok(())
}

/// Log the user's prompt response at the current time.
pub async fn log_prompt_response(
    ctx: &Context<'_>,
    similarity: f64,
) -> Result<()>
{
    let user = ctx.author().id.get();
    let percentage = similarity * 100.;
    let time = crate::now().as_secs();

    let db = ctx.data().db.lock().await;
    db.execute("
        INSERT INTO prompt_responses (user_id, time, accuracy)
        VALUES (?1, ?2, ?3)
    ", (user, time, percentage))?;

    Ok(())
}

pub async fn gooning_times(ctx: &Context<'_>, user: u64) -> Result<Vec<CumTime>> {
    internal::times(ctx, user, CumType::Gooning).await
}

pub async fn edging_times(ctx: &Context<'_>, user: u64) -> Result<Vec<CumTime>> {
    internal::times(ctx, user, CumType::Prejac).await
}