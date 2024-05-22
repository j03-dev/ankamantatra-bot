mod serializers;
#[cfg(test)]
mod test;

use rand::prelude::*;
use russenger::prelude::*;
use serializers::{load, Question};

#[action]
async fn Main(res: Res, req: Req) {
    res.send(GetStartedModel::new(Payload::default())).await;

    let quick_replies: Vec<QuickReply> = ["math", "science", "history", "sport", "programming"]
        .into_iter()
        .map(|k| {
            let payload = Payload::new(ChooseCategory, Some(Data::new(k, None)));
            QuickReply::new(k, "", payload)
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
    res.send(TextModel::new(&req.user, &question.question)).await; // send question
    let options = &question.options;
    let answer = &question.answer;

    let quick_replies = options
        .iter()
        .map(|opt| {
            let response = opt.to_string();
            let data = Data::new([&response, &answer.to_string()], None);
            let payload = Payload::new(ShowResponse, Some(data));
            QuickReply::new(&response, "", payload)
        })
        .collect::<Vec<_>>();
    let quick_reply_model = QuickReplyModel::new(&req.user, "Choose an option", quick_replies);
    res.send(quick_reply_model).await; // send options
}

#[action]
async fn ShowResponse(res: Res, req: Req) {
    let [response, answer]: [String; 2] = req.data.get_value();
    if response.to_lowercase() == answer.to_lowercase() {
        res.send(TextModel::new(&req.user, "Correct!")).await;
    } else {
        res.send(TextModel::new(&req.user, "Incorrect!")).await;
        res.send(TextModel::new(
            &req.user,
            &format!("The answer is : {answer}"),
        ))
        .await;
    }
    Main.execute(res, req).await;
}

russenger_app!(Main, ChooseCategory, ShowResponse);
