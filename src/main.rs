mod serializers;
#[cfg(test)]
mod test;

use russenger::prelude::*;
use serializers::{load, Category};

#[action]
async fn Main(res: Res, req: Req) {
    res.send(GetStartedModel::new(Payload::default())).await;
    let quick_reply = |k| {
        QuickReply::new(
            k,
            "",
            Payload::new(ChooseCategory, Some(Data::new(k, None))),
        )
    };
    let quick_replies: Vec<QuickReply> = vec![
        quick_reply("math"),
        quick_reply("science"),
        quick_reply("history"),
        quick_reply("sport"),
        quick_reply("programming"),
    ];

    let quick_reply_model = QuickReplyModel::new(&req.user, "Choose Category", quick_replies);
    res.send(quick_reply_model).await;
}

#[action]
async fn ChooseCategory(res: Res, req: Req) {
    let data = load().unwrap();
    let category: String = req.data.get_value();
    let _questions = match category.as_str() {
        "math" => data.math,
        "science" => data.science,
        "history" => data.history,
        "sport" => data.sports,
        "programming" => data.programming,
        _ => Category::default(),
    };
    todo!()
}

russenger_app!(Main, ChooseCategory);
