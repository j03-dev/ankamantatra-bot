use crate::models::{RussengerUser, User};
use russenger::prelude::*;

pub async fn migrate() -> Result<()> {
    let database = Database::new().await?;
    let conn = database.conn;
    migrate!([RussengerUser, User], &conn);
    Ok(())
}
