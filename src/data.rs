use std::{fs, io, collections::HashMap};

use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone)]
pub struct Thesis {
    pub l: String, //Question
    pub s: String, //Title
    pub x: String, //annotation
}
type Theses = HashMap<String, Thesis>;

#[derive(Serialize, Deserialize, Clone)]
pub struct List {
    pub name: String,   //Name
    pub name_x: String, //Short
}
type Lists = HashMap<String, List>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Answer {
    pub selection: String,
    pub statement: String,
}

type Answers = HashMap<String, HashMap<String, Answer>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Data {
    pub theses: Theses,
    pub lists: Lists,
    pub answers: Answers,
}

impl Default for Answer {
    fn default() -> Self {
        Answer {
            selection: "d".to_string(),
            statement: "".to_string(),
        }
    }
}

pub fn read_data() -> Result<Data, DataError> {
        Ok(serde_json::from_str(&fs::read_to_string("data.json")?)?)
}

#[derive(Debug, Error)]
pub enum DataError{
    #[error("failed to open 'data.json' file")]
    IoError(#[from] io::Error),
    #[error("could not parse 'data.json' file")]
    JsonError(#[from] serde_json::Error)
}
