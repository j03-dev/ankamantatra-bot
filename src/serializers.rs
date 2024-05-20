use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct QuestionMultiple {
    pub question: String,
    pub options: Vec<String>, // Changez Vec<i32> à Vec<String> pour uniformiser les options
    pub answer: Vec<String>,  // Changez Vec<i32> à Vec<String> pour uniformiser les réponses
}

#[derive(Deserialize, Debug)]
pub struct QuestionUnique {
    pub question: String,
    pub options: Vec<String>,
    pub answer: String,
}

#[derive(Deserialize, Debug)]
pub struct QuestionNumber {
    pub question: String,
    pub answer: i32,
}

#[derive(Deserialize, Debug)]
pub struct QuestionString {
    pub question: String,
    pub answer: String,
}

#[derive(Deserialize, Debug)]
pub struct Category {
    pub multiple: Option<Vec<QuestionMultiple>>,
    pub unique: Option<Vec<QuestionUnique>>,
    pub number: Option<Vec<QuestionNumber>>,
    pub string: Option<Vec<QuestionString>>,
}

#[derive(Deserialize, Debug)]
pub struct Quiz {
    pub math: Category,
    pub science: Category,
    pub history: Category,
    pub sports: Category,
    pub programming: Category,
}

pub fn load() -> Result<Quiz, String> {
    let data_string = std::fs::read_to_string("data.json").map_err(|err| err.to_string())?;
    serde_json::from_str(&data_string).map_err(|err| err.to_string())?
}
