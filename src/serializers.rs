use serde::Deserialize;
use serde_json::{from_str, Value};
use std::{
    fs::File,
    io::{self, Read},
};

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Question {
    pub question: String,
    pub options: Vec<Value>,
    pub answer: Value,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Data {
    pub math: Vec<Question>,
    pub science: Vec<Question>,
    pub history: Vec<Question>,
    pub sports: Vec<Question>,
    pub programming: Vec<Question>
}

pub fn load() -> io::Result<Data> {
    let mut output = String::new();
    let mut file = File::open("data.json")?;
    file.read_to_string(&mut output)?;
    from_str(&output).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
