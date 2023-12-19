use log::info;
use anyhow::Result;
use rusqlite::{OptionalExtension, types::Null};
use crate::Context;

const MIGRATIONS: [&str; 1] = [
    include_str!("../migrations/base.sql"),
];

/// Created by `db::in_action` whenever a user is doing something.
/// The user is considered `doing_something` until it is dropped.
/// Panics if any other `ActionGuards` exist
/// (i.e. `doing_something` is already 1).
pub struct ActionGuard<'a> {
    for_id: u64,
    db: &'a tokio::sync::Mutex<rusqlite::Connection>,
}

#[derive(Debug)]
pub struct Times {
    pub gooning: Vec<(u64, u64)>,
    pub prejac:  Vec<(u64, u64)>
}

enum TimeSort {
    Top,
    Recent,
}

impl<'a> Drop for ActionGuard<'a> {
    fn drop(&mut self) {
        tokio::task::block_in_place(|| {
            let db = self.db.blocking_lock();
            db.execute("UPDATE users SET doing_something = 0 where id = ?1",
                [self.for_id])
                .unwrap();
        });
    }
}

impl<'a> ActionGuard<'a> {
    pub async fn new(ctx: &Context<'a>) -> Result<ActionGuard<'a>> {
        let user_id = ctx.author().id.get();
        let db = ctx.data().db.lock().await;
        let changed = db
            .execute("UPDATE users SET doing_something = 1 WHERE id = ?1",
            [user_id])?;
        assert!(changed == 1);

        Ok(ActionGuard { for_id: user_id, db: &ctx.data().db, })
    }
}

pub fn migrations(db: &rusqlite::Connection) -> anyhow::Result<()> {
    for (idx, migration) in MIGRATIONS.iter().enumerate() {
        info!("migration {}: {:?}", idx, db.execute_batch(migration)?);
    }

    Ok(())
}

pub async fn log_prejac(ctx: &Context<'_>, start: u64, end: u64)
    -> Result<()>
{
    let db = ctx.data().db.lock().await;

    db.execute("
        INSERT INTO cum_times (user_id, started_at, ended_at, is_complete, what)
        VALUES      (?1, ?2, ?3, ?4, ?5)",
        (ctx.author().id.get(), start, end, true, "prejac"))?;
    
    Ok(())
}

pub async fn start_gooning(ctx: &Context<'_>) -> Result<()> {
    let now = crate::now().as_secs();
    let author = ctx.author().id.get();

    let db = ctx.data().db.lock().await;
    db.execute("
        INSERT INTO cum_times (user_id, started_at, ended_at, is_complete, what)
        VALUES      (?1, ?2, ?3, ?4, ?5)",
        (author, now, Null, false, "gooning"))?;

    db.execute("UPDATE users SET doing_something = 1 WHERE id = ?1",
        [author])?;

    Ok(())
}

pub async fn stop_gooning(ctx: &Context<'_>) -> Result<u64> {
    let now = crate::now().as_secs();
    let author = ctx.author().id.get();

    let db = ctx.data().db.lock().await;

    let result: u64 = db.query_row_and_then("
        UPDATE    cum_times
        SET       ended_at = ?1, is_complete = true
        WHERE     user_id = ?2 AND is_complete = false AND what = 'gooning'
        RETURNING ended_at, started_at",
        [now, ctx.author().id.get()],
        |row| anyhow::Ok(u64::saturating_sub(row.get(0)?, row.get(1)?)))?;
    
    db.execute("UPDATE users SET doing_something = 0 WHERE id = ?1",
        [author])?;

    Ok(result)
}

pub async fn doing_something(ctx: Context<'_>) -> Result<bool> {
    let result: Option<bool> = {
        let db = ctx.data().db.lock().await;

        db
            .query_row(
                "SELECT doing_something FROM users WHERE id = ?1",
                [ctx.author().id.get()],
                |row| row.get(0))
            .optional()?
    };

    match result {
        /* This user has never used Rk9 before */
        None => {
            add_user(&ctx).await?;
            Ok(false)
        },
        /* This user has used Rk9 before */
        Some(r) => Ok(r),
    }
}

/* Useful for command predicates where we cant just ! */
pub async fn not_doing_something(ctx: Context<'_>) -> Result<bool> {
    doing_something(ctx).await.map(|v| !v)
}

async fn add_user(ctx: &Context<'_>) -> Result<()> {
    let db = ctx.data().db.lock().await;

    db.execute("INSERT INTO users (id, doing_something) VALUES (?1, ?2)",
        (ctx.author().id.get(), false))?;
    Ok(())
}

/// Create, if one does not already exist, a server configuration entry.
async fn ensure_server_config_exists(ctx: &Context<'_>) -> anyhow::Result<()> {
    let guild = ctx
        .guild_id()
        .ok_or_else(|| anyhow::anyhow!("ensure_exists called on a DM context"))?
        .get();
    let db = ctx.data().db.lock().await;

    db.execute("INSERT OR IGNORE into server_configurations VALUES (?1, ?2)",
        (guild, Null))?;
    
    Ok(())
}

pub async fn set_post_channel(
    ctx: &Context<'_>,
    guild_id: u64,
    channel_id: Option<u64>,
) -> Result<()>
{
    ensure_server_config_exists(ctx).await?;

    let db = ctx.data().db.lock().await;

    db.execute("
        UPDATE server_configurations
        SET    post_channel = ?1
        WHERE  id = ?2",
        (channel_id, guild_id))?;

    Ok(())
}

pub async fn post_channel(
    ctx: &Context<'_>,
    guild_id: u64
) -> Result<Option<u64>>
{
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

pub async fn best_times(ctx: &Context<'_>) -> Result<Times> {
    times(ctx, TimeSort::Top).await
}

pub async fn recent_times(ctx: &Context<'_>) -> Result<Times> {
    times(ctx, TimeSort::Recent).await
}

/* This function is a piece of work. But it's sure DRY etc. */
async fn times(ctx: &Context<'_>, typ: TimeSort) -> Result<Times> {
    let user = ctx.author().id.get();
    let db = ctx.data().db.lock().await;

    let mut out = Times { gooning: vec![], prejac: vec![], };

    for category in ["gooning", "prejac"] {
        let sort = match typ {
            TimeSort::Top => "ended_at - started_at",
            TimeSort::Recent => "ended_at",
        };

        let order = match category {
            "gooning" => "DESC",
            "prejac"  => "ASC",
            _ => unreachable!(),
        };

        let mut stmt = db.prepare(&format!("
            SELECT   started_at, ended_at, what
            FROM     cum_times
            WHERE    user_id = ?1 AND is_complete = 1 AND what = ?2
            ORDER BY {sort} {order}
            LIMIT    3
        "))?;
        let mut rows = stmt.query((user, category))?;

        while let Some(row) = rows.next()? {
            let start: u64   = row.get(0)?;
            let end:   u64   = row.get(1)?;
            let what: String = row.get(2)?;

            match what.as_ref() {
                "gooning" => out.gooning.push((end, start)),
                "prejac"  => out.prejac.push( (end, start)),
                _ => unreachable!(),
            }
        }
    }

    Ok(out)
}