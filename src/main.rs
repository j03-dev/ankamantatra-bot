mod gemini;
mod migrate;
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

#[derive(Serialize, Deserialize, Default)]
enum Settings {
    #[default]
    ResetScoreAccount,
    DeleteAccount,
    ChooseCategory,
}

async fn index(res: Res, req: Req) -> Result<()> {
    res.send(GetStartedButtonModel::new(Payload::new("/home", None)))
        .await?;
    let payload = |setting| Payload::new("/setting", Some(Data::new(setting)));
    let persistent_menu = PersistentMenuModel::new(
        &req.user,
        vec![
            Button::Postback {
                title: "Reset Score".into(),
                payload: payload(Settings::ResetScoreAccount),
            },
            Button::Postback {
                title: "Delete Account".into(),
                payload: payload(Settings::DeleteAccount),
            },
            Button::Postback {
                title: "Change Category".into(),
                payload: payload(Settings::ChooseCategory),
            },
        ],
    );
    res.send(persistent_menu).await?;
    Ok(())
}

async fn home(res: Res, req: Req) -> Result<()> {
    if let Some(user) = User::get(kwargs!(user_id == &req.user), &req.query.conn).await? {
        let username = format!("username:{}", user.name);
        res.send(TextModel::new(&req.user, username)).await?;
        let score = format!("score:{}", user.score);
        res.send(TextModel::new(&req.user, score)).await?;
        ask_question(res, req).await?;
    } else {
        let message = "Please provide your pseudonym in this field.";
        res.send(TextModel::new(&req.user, message)).await?;
        res.redirect("/register").await?;
    }
    Ok(())
}

async fn setting(res: Res, req: Req) -> Result<()> {
    let conn = req.query.conn.clone();
    if let Some(mut user) = User::get(kwargs!(user_id == &req.user), &conn).await? {
        match req.data.get_value::<Settings>() {
            Settings::ResetScoreAccount => {
                user.score = 0;
                user.update(&conn).await?;
            }
            Settings::DeleteAccount => {
                user.delete(&conn).await?;
            }
            Settings::ChooseCategory => {
                let quick_replies = ["math", "science", "history", "sport", "programming"]
                    .into_iter()
                    .map(|category| {
                        QuickReply::new(
                            category,
                            None,
                            Payload::new("/choose_category", Some(Data::new(category))),
                        )
                    })
                    .collect();

                res.send(QuickReplyModel::new(
                    &req.user,
                    "Choose Category",
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
    let username: String = req.data.get_value();
    let result = User::create(kwargs!(name = &username, user_id = &req.user), conn).await;
    let message = match result {
        Ok(_) => "User registered successfully",
        Err(err) => &err.to_string(),
    };
    res.send(TextModel::new(&req.user, message)).await?;
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
            let message = "Invalid category";
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

    let quick_reply = |question_answer: QuestionAndAnswer| {
        QuickReply::new(
            question_answer.user_answer.clone(),
            None,
            Payload::new("/response", Some(Data::new(question_answer))),
        )
    };

    let quick_replies = options
        .iter()
        .map(|option| {
            quick_reply(QuestionAndAnswer {
                question: question.question.clone(),
                true_answer: true_answer.to_string(),
                user_answer: option.to_string(),
            })
        })
        .collect();

    let quick_reply = QuickReplyModel::new(&req.user, "Choose an option", quick_replies);
    res.send(quick_reply).await?;

    Ok(())
}

async fn response(res: Res, req: Req) -> Result<()> {
    let QuestionAndAnswer {
        question,
        user_answer,
        true_answer,
    } = req.data.get_value();
    let conn = req.query.conn.clone();
    if user_answer.to_lowercase() == true_answer.to_lowercase() {
        if let Some(mut user) = User::get(kwargs!(user_id == &req.user), &conn).await? {
            user.score += 1;
            user.update(&conn).await?;
        }
        res.send(TextModel::new(&req.user, "Correct!")).await?;
    } else {
        res.send(TextModel::new(&req.user, "Incorrect!")).await?;
        let message = format!("The answer is : {true_answer}");
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
    let category: String = req.data.get_value();
    let conn = &req.query.conn;
    if let Some(mut user) = User::get(kwargs!(user_id == &req.user), conn).await? {
        user.category = Some(category);
        user.update(conn).await?;
        res.send(TextModel::new(&req.user, "Category is set"))
            .await?;
    }
    ask_question(res, req).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    migrate::migrate().await?;

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
