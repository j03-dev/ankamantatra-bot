use russenger::prelude::*;

#[derive(Model, FromRow, Clone)]
pub struct User {
    #[model(primary_key = true, auto = true)]
    pub id: Integer,

    #[model(unique = true, size = 20)]
    pub name: String,

    #[model(unique = true, foreign_key = "RussengerUser.facebook_user_id")]
    pub user_id: String,

    #[model(default = 0)]
    pub score: Integer,

    #[model(size = 20, default = "math")]
    pub category: Option<String>,
}
