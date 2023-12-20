use rusqlite::types::Null;
use crate::Context;
use anyhow::Result;

#[derive(Clone)]
pub struct CumTime {
    pub started_at: u64,
    pub ended_at: u64,
    pub description: String,
    pub typ: CumType,
}

#[derive(Clone, Copy)]
pub enum CumType {
    Prejac,
    Gooning,
}

impl ToString for CumType {
    fn to_string(&self) -> String {
        match self {
            CumType::Gooning => String::from("gooning"),
            CumType::Prejac  => String::from("prejac"),
        }
    }
}

/// Creates a new user by the ID in the context. Does nothing if the user
/// already exists.
pub(super) async fn create_user(ctx: &Context<'_>) -> Result<()> {
    let db = ctx.data().db.lock().await;
    db.execute("
        INSERT OR IGNORE INTO users (id)
        VALUES (?1)",
        [ctx.author().id.get()])?;

    Ok(())
}

// TODO: Refactor these into their own crate so they can be used as proc
// macros on db functions.
/// Creates a new server configuration by the guild ID of the context. Does
/// nothing if it already exists.
/// Returns err when called in a DM context.
pub(super) async fn create_server_configuration(
    ctx: &Context<'_>,
    guild_id: u64
) -> anyhow::Result<()>
{
    let db = ctx.data().db.lock().await;
    db.execute("
        INSERT OR IGNORE into server_configurations
        VALUES (?1, ?2)",
        (guild_id, Null))?;
    
    Ok(())
}

/// Returns data about every time the user logged an edge / prejac.
/// Output is guaranteed to be sorted from longest to shortest goon sessions,
/// and from shortest to longest prejac sessions.
pub(super) async fn times(
    ctx: &Context<'_>,
    user: u64,
    typ: CumType,
) -> Result<Vec<CumTime>>
{
    let db = ctx.data().db.lock().await;
    let mut stmt = db.prepare("
        SELECT started_at, ended_at, description
        FROM cum_times
        WHERE user_id = ?1 AND is_complete = true AND what = ?2
    ")?;

    let mut rows = stmt.query((user, typ.to_string()))?;
    let mut out = Vec::new();

    while let Some(row) = rows.next()? {
        out.push(CumTime {
            started_at:  row.get(0)?,
            ended_at:    row.get(1)?,
            description: row.get(2)?,
            typ,
        });
    }

    match typ {
        CumType::Gooning =>
            out.sort_by(|a, b| (a.ended_at - a.started_at).cmp(&(b.ended_at - b.started_at))),
        CumType::Prejac =>
            out.sort_by(|a, b| (b.ended_at - b.started_at).cmp(&(a.ended_at - a.started_at))),
    }

    Ok(out)
}