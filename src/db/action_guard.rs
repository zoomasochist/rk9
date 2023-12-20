/// Created by `db::in_action` whenever a user is doing something.
/// The user is considered `doing_something` until it is dropped.
/// Panics if any other `ActionGuards` exist
/// (i.e. `doing_something` is already 1).
pub struct ActionGuard<'a> {
    for_id: u64,
    db: &'a tokio::sync::Mutex<rusqlite::Connection>,
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
    pub async fn new(
        ctx: &crate::Context<'a>
    ) -> anyhow::Result<ActionGuard<'a>>
    {
        let user_id = ctx.author().id.get();
        let db = ctx.data().db.lock().await;
        let changed = db
            .execute("UPDATE users SET doing_something = 1 WHERE id = ?1",
            [user_id])?;
        assert!(changed == 1);

        Ok(ActionGuard { for_id: user_id, db: &ctx.data().db, })
    }
}