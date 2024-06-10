use rusql_alchemy::prelude::*;

#[derive(Debug, Deserialize, Model)]
pub struct Score {
    #[model(primary_key = true, auto = true)]
    pub id: Integer,
    #[model(unique = true, null = false, size = 20)]
    pub name: String,
    #[model(
        unique = true,
        null = false,
        foreign_key = "RussengerUser.facebook_user_id"
    )]
    pub user_id: String,
    #[model(default = 0)]
    pub score: Integer,
}
