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
        .map(|category| {
            let payload = Payload::new(ChooseCategory, Some(Data::new(category, None))); // send to Choosecategory action the category
            QuickReply::new(category, "", payload)
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
    res.send(TextModel::new(&req.user, &question.question))
        .await; // send question
    let options = &question.options;
    let real_answer = &question.answer;

    let quick_replies = options
        .iter()
        .map(|option| {
            let possible_answer = option.to_string();
            let value = [&possible_answer, &real_answer.to_string()]; // the value to send to ShowResponse action
            let data = Data::new(value, None);
            let payload = Payload::new(ShowResponse, Some(data));
            QuickReply::new(&possible_answer, "", payload)
        })
        .collect::<Vec<_>>();
    let quick_reply_model = QuickReplyModel::new(&req.user, "Choose an option", quick_replies);
    res.send(quick_reply_model).await; // send options
}

#[action]
async fn ShowResponse(res: Res, req: Req) {
    let [user_answer, answer]: [String; 2] = req.data.get_value();
    if user_answer.to_lowercase() == answer.to_lowercase() {
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
