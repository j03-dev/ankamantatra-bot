mod gemini;
mod serializers;
#[cfg(test)]
mod test;

use gemini::ask_gemini;
use rand::prelude::*;
use russenger::{models::RussengerUser, prelude::*, App};
use serde::{Deserialize, Serialize};
use serializers::load;

#[derive(Model, FromRow, Clone)]
pub struct UserAccount {
    #[model(primary_key = true, auto = true)]
    pub id: Integer,

    #[model(unique = true, size = 20)]
    pub name: String,

    #[model(unique = true, foreign_key = "RussengerUser.facebook_user_id")]
    pub user_id: String,

    #[model(default = 0)]
    pub score: Integer,
}

#[derive(Serialize, Deserialize, Default)]
enum Settings {
    #[default]
    ResetScoreAccount,
    DeleteAccount,
}

#[action]
async fn index(res: Res, req: Req) -> Result<()> {
    res.send(GetStartedButtonModel::new(Payload::default()))
        .await?;

    res.send(PersistentMenuModel::new(
        &req.user,
        vec![
            Button::Postback {
                title: "Reset Score".into(),
                payload: Payload::new(
                    "/setting",
                    Some(Data::new(Settings::ResetScoreAccount, None)),
                ),
            },
            Button::Postback {
                title: "Delete Account".into(),
                payload: Payload::new("/setting", Some(Data::new(Settings::DeleteAccount, None))),
            },
        ],
    ))
    .await?;

    match UserAccount::get(kwargs!(user_id == &req.user), &req.query.conn).await {
        Some(user_account) => {
            let username = format!("username:{}", user_account.name);
            res.send(TextModel::new(&req.user, &username)).await?;
            let score = format!("score:{}", user_account.score);
            res.send(TextModel::new(&req.user, &score)).await?;
        }
        None => {
            let message = "Please provide your pseudonym in this field.";
            res.send(TextModel::new(&req.user, message)).await?;
            req.query.set_path(&req.user, "/register").await;
            return Ok(());
        }
    }

    let quick_reply = |c| {
        QuickReply::new(
            c,
            None,
            Payload::new("/choose_category", Some(Data::new(c, None))),
        )
    };

    let quick_replies = ["math", "science", "history", "sport", "programming"]
        .into_iter()
        .map(quick_reply)
        .collect();

    let quick_reply_model = QuickReplyModel::new(&req.user, "Choose Category", quick_replies);
    res.send(quick_reply_model).await?;
    Ok(())
}

#[action]
async fn action_setting(res: Res, req: Req) -> Result<()> {
    let conn = req.query.conn.clone();
    if let Some(mut user_account) = UserAccount::get(kwargs!(user_id == &req.user), &conn).await {
        match req.data.get_value::<Settings>() {
            Settings::ResetScoreAccount => {
                user_account.score = 0;
                user_account.update(&conn).await;
            }
            Settings::DeleteAccount => {
                user_account.delete(&conn).await;
            }
        };
    }
    index(res, req).await?;
    Ok(())
}

#[action]
async fn register(res: Res, req: Req) -> Result<()> {
    let username: String = req.data.get_value();
    let message = if UserAccount::create(
        kwargs!(name = &username, user_id = &req.user),
        &req.query.conn,
    )
    .await
    {
        "User registered successfully"
    } else {
        "Failed to register user"
    };
    res.send(TextModel::new(&req.user, message)).await?;
    index(res, req).await?;
    Ok(())
}

#[derive(Serialize, Deserialize, Default)]
struct QuestionAndAnswer {
    question: String,
    user_anwswer: String,
    true_answer: String,
}

#[action]
async fn choose_category(res: Res, req: Req) -> Result<()> {
    let data = match load() {
        Ok(data) => data,
        Err(err) => {
            let message = "Failed to load categories";
            res.send(TextModel::new(&req.user, message)).await?;
            error::bail!("{err:?}");
        }
    };

    let category: String = req.data.get_value();
    let questions = match category.as_str() {
        "math" => data.math,
        "science" => data.science,
        "history" => data.history,
        "sport" => data.sports,
        "programming" => data.programming,
        _ => {
            let message = "Invalid category";
            res.send(TextModel::new(&req.user, message)).await?;
            return Ok(());
        }
    };

    let index = rand::thread_rng().gen_range(0..questions.len());
    let question = &questions[index];

    res.send(TextModel::new(&req.user, &question.question))
        .await?;

    let options = &question.options;
    let true_answer = &question.answer;

    let quick_reply = |qa: QuestionAndAnswer| {
        QuickReply::new(
            &qa.user_anwswer.clone(),
            None,
            Payload::new("/response", Some(Data::new(qa, None))),
        )
    };

    let quick_replies = options
        .iter()
        .map(|option| {
            quick_reply(QuestionAndAnswer {
                question: question.question.clone(),
                true_answer: true_answer.to_string(),
                user_anwswer: option.to_string(),
            })
        })
        .collect();

    let quick_reply_model = QuickReplyModel::new(&req.user, "Choose an option", quick_replies);

    res.send(quick_reply_model).await?;

    Ok(())
}

#[action]
async fn show_response(res: Res, req: Req) -> Result<()> {
    let QuestionAndAnswer {
        question,
        user_anwswer,
        true_answer,
    } = req.data.get_value();
    let conn = req.query.conn.clone();
    if user_anwswer.to_lowercase() == true_answer.to_lowercase() {
        if let Some(mut user_account) = UserAccount::get(kwargs!(user_id == &req.user), &conn).await
        {
            user_account.score += 1;
            user_account.update(&conn).await;
        }
        res.send(TextModel::new(&req.user, "Correct!")).await?;
    } else {
        res.send(TextModel::new(&req.user, "Incorrect!")).await?;
        let message = format!("The answer is : {true_answer}");
        res.send(TextModel::new(&req.user, &message)).await?;
        let prompt = format!("The question is {question}, explain to me why: {true_answer} is the right answer, in one paragraph");
        let response = ask_gemini(prompt).await?;

        if let Some(candidate) = response.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                res.send(TextModel::new(&req.user, &part.text)).await?;
            }
        }
    }

    index(res, req).await?;
    Ok(())
}

#[russenger::main]
async fn main() -> error::Result<()> {
    let database = Database::new().await?;
    let conn = database.conn;
    migrate!([RussengerUser, UserAccount], &conn);

    let mut app = App::init().await?;
    app.add("/", index).await;
    app.add("/register", register).await;
    app.add("/setting", action_setting).await;
    app.add("/choose_category", choose_category).await;
    app.add("/response", show_response).await;

    russenger::launch(app).await?;
    Ok(())
}
