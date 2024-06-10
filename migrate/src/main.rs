use models::Score;
use rusql_alchemy::prelude::*;

#[tokio::main]
async fn main() {
    let conn = config::db::Database::new().await.conn;
    migrate!([Score], &conn);
}
