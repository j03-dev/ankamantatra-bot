mod gemini;
mod models;
mod serializers;
#[cfg(test)]
mod test;

use error::Context;
use gemini::ask_gemini;
use rand::prelude::*;
use russenger::{prelude::*, App};
use serde::{Deserialize, Serialize};
use serializers::load;

use crate::models::User;

#[derive(Serialize, Deserialize)]
enum Settings {
    ResetScoreAccount,
    DeleteAccount,
    ChooseCategory,
}

async fn index(res: Res, req: Req) -> Result<()> {
    let wellcome_message = r#"
    👋 Welcome to Boto Chat, your interactive quiz companion!
    🎮 Get ready to test your knowledge across various categories and have fun learning! 🌟
    "#;
    res.send(TextModel::new(&req.user, wellcome_message))
        .await?;
    res.send(QuickReplyModel::new(
        &req.user,
        "GetStarted",
        [QuickReply::new("Start", None, Payload::new("/home", None))],
    ))
    .await?;
    res.send(GetStartedButtonModel::new(Payload::default()))
        .await?;
    let payload = |setting| Payload::new("/setting", Some(Data::new(setting)));
    let persistent_menu = PersistentMenuModel::new(
        &req.user,
        [
            Button::Postback {
                title: "🔄 Reset Score",
                payload: payload(Settings::ResetScoreAccount),
            },
            Button::Postback {
                title: "🗑️ Delete Account",
                payload: payload(Settings::DeleteAccount),
            },
            Button::Postback {
                title: "🔧 Change Category",
                payload: payload(Settings::ChooseCategory),
            },
        ],
    );
    res.send(persistent_menu).await?;
    Ok(())
}

async fn home(res: Res, req: Req) -> Result<()> {
    if let Some(user) = User::get(kwargs!(user_id == &req.user), &req.query.conn).await? {
        let username = format!("👤 Username: {}", user.name);
        res.send(TextModel::new(&req.user, username)).await?;
        let score = format!("🏆 Score: {}", user.score);
        res.send(TextModel::new(&req.user, score)).await?;
        ask_question(res, req).await?;
    } else {
        let message = "📝 Please provide your pseudonym in this field.";
        res.send(TextModel::new(&req.user, message)).await?;
        res.redirect("/register").await?;
    }
    Ok(())
}

async fn setting(res: Res, req: Req) -> Result<()> {
    let conn = req.query.conn.clone();
    if let Some(mut user) = User::get(kwargs!(user_id == &req.user), &conn).await? {
        match req.data.get_value::<Settings>()? {
            Settings::ResetScoreAccount => {
                user.score = 0;
                user.update(&conn).await?;
                res.send(TextModel::new(&req.user, "🔄 Score reset successfully!"))
                    .await?;
            }
            Settings::DeleteAccount => {
                user.delete(&conn).await?;
                res.send(TextModel::new(
                    &req.user,
                    "🗑️ Account deleted successfully!",
                ))
                .await?;
            }
            Settings::ChooseCategory => {
                let quick_replies = [
                    "🔢 Math",
                    "🔬 Science",
                    "📜 History",
                    "⚽ Sport",
                    "💻 Programming",
                ]
                .into_iter()
                .map(|category| {
                    QuickReply::new(
                        category,
                        None,
                        Payload::new("/choose_category", Some(Data::new(category))),
                    )
                });

                res.send(QuickReplyModel::new(
                    &req.user,
                    "🔧 Choose a Category",
                    quick_replies,
                ))
                .await?;
                return Ok(());
            }
        };
    }
    home(res, req).await?;
    Ok(())
}

async fn register(res: Res, req: Req) -> Result<()> {
    let conn = &req.query.conn;
    let username: String = req.data.get_value()?;
    User::create(kwargs!(name = &username, user_id = &req.user), conn).await?;
    res.send(TextModel::new(
        &req.user,
        "🎉 User registered successfully!",
    ))
    .await?;
    home(res, req).await?;
    Ok(())
}

#[derive(Serialize, Deserialize, Default)]
struct QuestionAndAnswer {
    question: String,
    user_answer: String,
    true_answer: String,
}

async fn ask_question(res: Res, req: Req) -> Result<()> {
    let data = load()?;
    let user = User::get(kwargs!(user_id = &req.user), &req.query.conn)
        .await?
        .context("failed to get user")?;

    let questions = match user.category.context("category not found")?.as_str() {
        "math" => data.math,
        "science" => data.science,
        "history" => data.history,
        "sport" => data.sports,
        "programming" => data.programming,
        _ => {
            let message = "🚫 Invalid category";
            res.send(TextModel::new(&req.user, message)).await?;
            return Ok(());
        }
    };

    let index = thread_rng().gen_range(0..questions.len());
    let question = &questions[index];

    res.send(TextModel::new(&req.user, &question.question))
        .await?;

    let options = &question.options;
    let true_answer = &question.answer;

    let quick_replies = options.into_iter().map(|option| {
        QuickReply::new(
            option,
            None,
            Payload::new(
                "/response",
                Some(Data::new(QuestionAndAnswer {
                    question: question.question.clone(),
                    user_answer: option.to_string(),
                    true_answer: true_answer.to_string(),
                })),
            ),
        )
    });

    let quick_reply = QuickReplyModel::new(&req.user, "🔍 Choose an option", quick_replies);
    res.send(quick_reply).await?;

    Ok(())
}

async fn response(res: Res, req: Req) -> Result<()> {
    let QuestionAndAnswer {
        question,
        user_answer,
        true_answer,
    } = req.data.get_value()?;
    let conn = req.query.conn.clone();
    if user_answer.to_lowercase() == true_answer.to_lowercase() {
        if let Some(mut user) = User::get(kwargs!(user_id == &req.user), &conn).await? {
            user.score += 1;
            user.update(&conn).await?;
        }
        res.send(TextModel::new(&req.user, "🎉 Correct!")).await?;
    } else {
        res.send(TextModel::new(&req.user, "❌ Incorrect!")).await?;
        let message = format!("🔍 The correct answer is: {true_answer}");
        res.send(TextModel::new(&req.user, message)).await?;
        let prompt = format!(
            r#"
            The question is {question},
            explain to me why: {true_answer}
            is the right answer, in one paragraph"#
        );
        let response = ask_gemini(prompt).await?;

        if let Some(candidate) = response.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                res.send(TextModel::new(&req.user, &part.text)).await?;
            }
        }
    }

    home(res, req).await?;
    Ok(())
}

async fn choose_category(res: Res, req: Req) -> Result<()> {
    let category: String = req.data.get_value()?;
    let conn = &req.query.conn;
    if let Some(mut user) = User::get(kwargs!(user_id == &req.user), conn).await? {
        user.category = Some(category);
        user.update(conn).await?;
        res.send(TextModel::new(&req.user, "🔧 Category is set!"))
            .await?;
    }
    ask_question(res, req).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    App::init()
        .await?
        .attach(
            Router::new()
                .add("/", index)
                .add("/home", home)
                .add("/register", register)
                .add("/setting", setting)
                .add("/choose_category", choose_category)
                .add("/ask_question", ask_question)
                .add("/response", response),
        )
        .launch()
        .await?;

    Ok(())
}
