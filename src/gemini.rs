use russenger::error::Result;
use serde::Deserialize;
use serde::Serialize;

const _URL: &str =
    "https://generativelanguage.googleapis.com/v1/models/gemini-pro:generateContent?key=";

#[derive(Serialize, Deserialize, Clone)]
pub struct Part {
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Content {
    role: String,
    pub parts: Vec<Part>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Candidate {
    pub content: Content,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Response {
    pub candidates: Vec<Candidate>,
}

pub(crate) async fn _ask_gemini(text: String) -> Result<Response> {
    let api_key = std::env::var("API_KEY").expect("pls check your env file");
    let api_url = format!("{_URL}{api_key}");
    let body = serde_json::json!(
        {
            "contents": [
                Content {
                    role: "user".to_owned(),
                    parts: vec![ Part { text } ]
                }
            ]
        }
    );
    let response = reqwest::Client::new()
        .post(api_url)
        .json(&body)
        .send()
        .await?;

    Ok(response.json().await?)
}
