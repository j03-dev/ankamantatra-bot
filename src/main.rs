mod gemini;
mod serializers;
#[cfg(test)]
mod test;

use gemini::ask_gemini;
use rand::prelude::*;
use russenger::error::Context;
use russenger::{models::RussengerUser, prelude::*};
use serializers::{load, Question};

#[derive(Model, FromRow, Clone)]
pub struct Score {
    #[model(primary_key = true)]
    pub id: Serial,
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

#[action]
async fn Main(res: Res, req: Req) {
    // res.send(GetStartedModel::new(Payload::default())).await?;

    // check if user has a score
    match Score::get(kwargs!(user_id == &req.user), &req.query.conn).await {
        Some(score) => {
            let message = format!("Your score is {}: {}", score.name, score.score);
            res.send(TextModel::new(&req.user, &message)).await?;
        }
        None => {
            // if user has no score, register user
            let message = "Please provide your pseudonym in this field.";
            res.send(TextModel::new(&req.user, message)).await?;
            req.query.set_action(&req.user, RegisterUser).await;
            return Ok(());
        }
    }

    let payload = |c| Payload::new(ChooseCategory, Some(Data::new(c, None)));

    // send quick replies of categories
    let quick_replies: Vec<QuickReply> = ["math", "science", "history", "sport", "programming"]
        .into_iter()
        .map(|category| QuickReply::new(category, "", payload(category)))
        .collect();

    let quick_reply_model = QuickReplyModel::new(&req.user, "Choose Category", quick_replies);
    res.send(quick_reply_model)
        .await
        .context("Failed to send quick replies")?;
    Ok(())
}

#[action]
async fn RegisterUser(res: Res, req: Req) {
    let username: String = req.data.get_value();
    let is_create = Score::create(
        kwargs!(name = &username, user_id = &req.user),
        &req.query.conn,
    )
    .await;
    let message = if is_create {
        "User registered successfully"
    } else {
        "Failed to register user"
    };
    res.send(TextModel::new(&req.user, message)).await?;
    Main.execute(res, req).await?;
    Ok(())
}

#[action]
async fn ChooseCategory(res: Res, req: Req) {
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
        "math" => &data.math,
        "science" => &data.science,
        "history" => &data.history,
        "sport" => &data.sports,
        "programming" => &data.programming,
        _ => {
            let message = "Invalid category";
            res.send(TextModel::new(&req.user, message)).await?;
            return Ok(());
        }
    };

    let index = rand::thread_rng().gen_range(0..questions.len());
    let question = &questions[index];

    send_question(res, req, question).await?;

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct DataValue {
    question: String,
    possible_answer: String,
    true_answer: String,
}

async fn send_question(res: Res, req: Req, question: &Question) -> error::Result<()> {
    res.send(TextModel::new(&req.user, &question.question))
        .await?; // send question
    let options = &question.options;
    let true_answer = &question.answer;

    let payload = |value| Payload::new(ShowResponse, Some(Data::new(value, None)));

    let quick_replies = options
        .iter()
        .map(|option| {
            let possible_answer = option.to_string();
            let _value = DataValue {
                question: question.question.clone(),
                possible_answer: possible_answer.clone(),
                true_answer: true_answer.to_string(),
            };
            QuickReply::new(&possible_answer, "", payload("test"))
        })
        .collect::<Vec<_>>();
    let quick_reply_model = QuickReplyModel::new(&req.user, "Choose an option", quick_replies);
    res.send(quick_reply_model).await?;
    Ok(())
}

#[action]
async fn ShowResponse(res: Res, req: Req) {
    let DataValue {
        question,
        possible_answer,
        true_answer,
    } = req.data.get_value();
    let conn = req.query.conn.clone();
    if possible_answer.to_lowercase() == true_answer.to_lowercase() {
        // increment score
        if let Some(mut score) = Score::get(kwargs!(user_id == &req.user), &conn).await {
            score.score += 1;
            score.update(&conn).await;
        }
        res.send(TextModel::new(&req.user, "Correct!")).await?;
    } else {
        res.send(TextModel::new(&req.user, "Incorrect!")).await?;
        let message = format!("The answer is : {true_answer}");
        res.send(TextModel::new(&req.user, &message)).await?;
        let prompt = format!("The question is {question}, explain to me why: {true_answer} is the right answer, in one paragraph");
        let response = ask_gemini(prompt).await.unwrap();

        for part in response.candidates[0].content.parts.iter() {
            res.send(TextModel::new(&req.user, &part.text)).await?;
        }
    }
    Main.execute(res, req).await?;

    Ok(())
}

#[russenger::main]
async fn main() -> error::Result<()> {
    let conn = Database::new().await.conn;
    migrate!([RussengerUser, Score], &conn);
    russenger::actions![Main, RegisterUser, ChooseCategory, ShowResponse];
    russenger::launch().await?;
    Ok(())
}
