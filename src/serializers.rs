use serde::Deserialize;
use serde_json::{from_str, Value};
use std::{
    fs::File,
    io::{self, Read},
};

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Question {
    pub question: String,
    pub options: Option<Vec<Value>>,
    pub answer: Option<Value>, // Using serde_json::Value to handle both Vec<String> and String
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Category {
    pub multiple: Option<Vec<Question>>,
    pub unique: Option<Vec<Question>>,
    pub number: Option<Vec<Question>>,
    pub string: Option<Vec<Question>>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Data {
    pub math: Category,
    pub science: Category,
    pub history: Category,
    pub sports: Category,
    pub programming: Category,
}

pub fn load() -> io::Result<Data> {
    let mut output = String::new();
    let mut file = File::open("data.json")?;
    file.read_to_string(&mut output)?;
    from_str(&output).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
