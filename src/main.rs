mod serializers;
#[cfg(test)]
mod test;

use rand::prelude::*;
use rand::rngs::StdRng;
use russenger::prelude::*;
use serializers::{load, Question};
use std::sync::Arc;
use tokio::sync::Mutex;

#[action]
async fn Main(res: Res, req: Req) {
    res.send(GetStartedModel::new(Payload::default())).await;

    let quick_replies: Vec<QuickReply> = vec!["math", "science", "history", "sport", "programming"]
        .into_iter()
        .map(|k| {
            QuickReply::new(
                k,
                "",
                Payload::new(ChooseCategory, Some(Data::new(k, None))),
            )
        })
        .collect();

    let quick_reply_model = QuickReplyModel::new(&req.user, "Choose Category", quick_replies);
    res.send(quick_reply_model).await;
}

#[action]
async fn ChooseCategory(res: Res, req: Req) {
    let data = match load() {
        Ok(data) => data,
        Err(err) => {
            res.send(TextModel::new(&req.user, "Failed to load categories"))
                .await;
            eprintln!("Error loading data: {:?}", err);
            return;
        }
    };

    let category: String = req.data.get_value();
    let category_data = match category.as_str() {
        "math" => &data.math,
        "science" => &data.science,
        "history" => &data.history,
        "sport" => &data.sports,
        "programming" => &data.programming,
        _ => {
            res.send(TextModel::new(&req.user, "Invalid category"))
                .await;
            return;
        }
    };

    let items = ["multiple", "unique", "number", "string"];
    let rng = Arc::new(Mutex::new(StdRng::from_entropy()));
    let chosen_item = {
        let mut rng = rng.lock().await;
        items.choose(&mut *rng)
    };

    match chosen_item {
        Some(&"multiple") => send_question(res, req, &category_data.multiple).await,
        Some(&"unique") => send_question(res, req, &category_data.unique).await,
        Some(&"number") => send_question(res, req, &category_data.number).await,
        Some(&"string") => send_question(res, req, &category_data.string).await,
        _ => {
            res.send(TextModel::new(&req.user, "No questions available"))
                .await;
        }
    };
}

async fn send_question(res: Res, req: Req, q: &Option<Vec<Question>>) {
    if let Some(questions) = q {
        let rng = Arc::new(Mutex::new(StdRng::from_entropy()));
        let (question, options, answer) = {
            let mut rng = rng.lock().await;
            let question = questions.choose(&mut *rng).unwrap();
            (
                question.question.clone(),
                question.options.clone(),
                question.answer.clone(),
            )
        };
        res.send(TextModel::new(&req.user, &question)).await;
        let quick_replies = options
            .unwrap()
            .iter()
            .map(|opt| {
                let response = opt.to_string();
                let answer = answer.clone().unwrap().to_string();
                QuickReply::new(
                    &response,
                    "",
                    Payload::new(ShowResponse, Some(Data::new([&response, &answer], None))),
                )
            })
            .collect();
        let quick_reply_model = QuickReplyModel::new(&req.user, "Choose an option", quick_replies);
        res.send(quick_reply_model).await;
    } else {
        res.send(TextModel::new(&req.user, "No questions available"))
            .await;
    }
}

#[action]
async fn ShowResponse(res: Res, req: Req) {
    let [resp, answ]: [String; 2] = req.data.get_value();
    if resp == answ {
        res.send(TextModel::new(&req.user, "Correct!")).await;
    } else {
        res.send(TextModel::new(&req.user, "Incorrect!")).await;
    }
}

russenger_app!(Main, ChooseCategory, ShowResponse);
