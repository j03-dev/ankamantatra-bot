mod gemini;
mod serializers;
#[cfg(test)]
mod test;

use gemini::ask_gemini;
use rusql_alchemy::prelude::*;

use rand::prelude::*;
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
    res.send(GetStartedModel::new(Payload::default())).await;

    let conn = req.query.conn.clone();

    match Score::get(kwargs!(user_id = &req.user), &conn).await {
        Some(score) => {
            res.send(TextModel::new(
                &req.user,
                &format!("Your score is {}: {}", score.name, score.score),
            ))
            .await;
        }
        None => {
            res.send(TextModel::new(
                &req.user,
                "Please provide your pseudonym in this field.",
            ))
            .await;
            req.query.set_action(&req.user, RegisterUser).await;
            return;
        }
    }

    let quick_replies: Vec<QuickReply> = ["math", "science", "history", "sport", "programming"]
        .into_iter()
        .map(|category| {
            let payload = Payload::new(ChooseCategory, Some(Data::new(category, None))); // send to Choosecategory action the category
            QuickReply::new(category, "", payload)
        })
        .collect();

    let quick_reply_model = QuickReplyModel::new(&req.user, "Choose Category", quick_replies);
    res.send(quick_reply_model).await;
}

#[action]
async fn RegisterUser(res: Res, req: Req) {
    let username: String = req.data.get_value();
    let message = if Score::create(
        kwargs!(name = &username, user_id = &req.user),
        &req.query.conn,
    )
    .await
    {
        "User registered successfully"
    } else {
        "Failed to register user"
    };

    res.send(TextModel::new(&req.user, message)).await;

    Main.execute(res, req).await;
}

#[action]
async fn ChooseCategory(res: Res, req: Req) {
    let data = match load() {
        Ok(data) => data,
        Err(err) => {
            let message = "Failed to load categories";
            res.send(TextModel::new(&req.user, message)).await;
            eprintln!("Error loading data: {:?}", err);
            return;
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
            res.send(TextModel::new(&req.user, message)).await;
            return;
        }
    };

    let index = rand::thread_rng().gen_range(0..questions.len());
    let question = &questions[index];

    send_question(res, req, question).await;
}

async fn send_question(res: Res, req: Req, question: &Question) {
    res.send(TextModel::new(&req.user, &question.question))
        .await; // send question
    let options = &question.options;
    let real_answer = &question.answer;

    let quick_replies = options
        .iter()
        .map(|option| {
            let possible_answer = option.to_string();
            let value = [
                &question.question,
                &possible_answer,
                &real_answer.to_string(),
            ];
            let data = Data::new(value, None);
            let payload = Payload::new(ShowResponse, Some(data));
            QuickReply::new(&possible_answer, "", payload)
        })
        .collect::<Vec<_>>();
    let quick_reply_model = QuickReplyModel::new(&req.user, "Choose an option", quick_replies);
    res.send(quick_reply_model).await;
}

#[action]
async fn ShowResponse(res: Res, req: Req) {
    let [question, user_answer, answer]: [String; 3] = req.data.get_value();
    let conn = req.query.conn.clone();
    if user_answer.to_lowercase() == answer.to_lowercase() {
        if let Some(score) = Score::get(kwargs!(user_id = &req.user), &conn).await {
            Score {
                score: score.score + 1,
                ..score
            }
            .update(&conn)
            .await;
        }
        res.send(TextModel::new(&req.user, "Correct!")).await;
    } else {
        res.send(TextModel::new(&req.user, "Incorrect!")).await;

        res.send(TextModel::new(
            &req.user,
            &format!("The answer is : {answer}"),
        ))
        .await;

        let response = ask_gemini(format!(
            "The question is {question}, explain to me why: {answer} is the right answer, in one paragraph"
        ))
        .await
        .unwrap();

        for part in response.candidates[0].content.parts.clone() {
            res.send(TextModel::new(&req.user, &part.text)).await;
        }
    }
    Main.execute(res, req).await;
}

#[russenger::main]
async fn main() {
    let conn = Database::new().await.conn;
    migrate!([RussengerUser, Score], &conn);
    russenger::actions![Main, RegisterUser, ChooseCategory, ShowResponse];
    russenger::launch().await;
}