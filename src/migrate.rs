use crate::models::{RussengerUser, UserAccount};
use russenger::prelude::*;

pub async fn migrate() -> Result<()> {
    let database = Database::new().await?;
    let conn = database.conn;
    migrate!([RussengerUser, UserAccount], &conn);
    Ok(())
}
