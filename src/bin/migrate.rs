use ankamantatra_bot::models::{RussengerUser, UserAccount};
use russenger::prelude::*;

#[russenger::main]
async fn main() -> Result<()> {
    let database = Database::new().await?;
    let conn = database.conn;
    migrate!([RussengerUser, UserAccount], &conn);
    Ok(())
}
