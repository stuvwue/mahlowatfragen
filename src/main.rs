use std::error::Error;
use warp::{self, Filter};

use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;

#[derive(Serialize, Deserialize, Clone)]
pub struct Thesis {
    pub l: String, //Question
    pub s: String, //Title
    pub x: String, //annotation
}
type Theses = BTreeMap<String, Thesis>;

#[derive(Serialize, Deserialize, Clone)]
pub struct List {
    pub name: String,   //Name
    pub name_x: String, //Short
}
type Lists = BTreeMap<String, List>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Answer {
    pub selection: String,
    pub statement: String,
}

type Answers = BTreeMap<String, BTreeMap<String, Answer>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
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

pub fn as_form(id: &str, thesis: &Thesis, answer: &Answer) -> String {
    format!(
        r#"<h2> {title} </h2>
    {thesis}<br>
  <input type="radio" name="{id}selection" value="a" {approve}> Zustimmung
  <input type="radio" name="{id}selection" value="b" {dunno}> Egal
  <input type="radio" name="{id}selection" value="c" {nope}> Ablehnung
  <input type="radio" name="{id}selection" value="d" {skip}> Überspringen<br>
  Begründung:<br>
  <input type="text" name="{id}statement" value="{statement}"><br>
  <input type="submit" value="Änderungen speichern">"#,
        thesis = thesis.l,
        title = thesis.s,
        id = id,
        approve = if answer.selection == "a" {
            "checked"
        } else {
            ""
        },
        dunno = if answer.selection == "b" {
            "checked"
        } else {
            ""
        },
        nope = if answer.selection == "c" {
            "checked"
        } else {
            ""
        },
        skip = if answer.selection == "d" {
            "checked"
        } else {
            ""
        },
        statement = answer.statement,
    )
}

#[tokio::main]
async fn main() {
    let form =
        warp::get().and(warp::path::param()).map(
            move |list_id: String| match read_and_format_forms(&list_id) {
                Ok(forms) => warp::reply::html(forms),
                Err(err) => warp::reply::html(format!("{}", err)),
            },
        );

    let reply = warp::post()
        .and(warp::path::param())
        .and(warp::path("send"))
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::form())
        .map(|list_id: String, form: HashMap<String, String>| {
            let mut answers: BTreeMap<String, Answer> = BTreeMap::new();
            for (key, value) in form.into_iter() {
                let (id, field) = key.split_at(1);
                let mut answer = answers.entry(id.to_string()).or_default();
                if field == "selection" {
                    (*answer).selection = value.to_string();
                } else if field == "statement" {
                    (*answer).statement = value.to_string();
                }
            }
            let mut config: Config = {
                let file = File::open("data.json").unwrap();
                let reader = BufReader::new(file);
                serde_json::from_reader(reader).unwrap()
            };

            config.answers.insert(list_id, answers);
            let file = File::create("data.json").unwrap();
            let writer = BufWriter::new(file);
            serde_json::to_writer_pretty(writer, &config).unwrap();
            Ok("answers updated")
        });

    warp::serve(form.or(reply))
        .run(([127, 0, 0, 1], 3030))
        .await;
}

fn read_and_format_forms(list_id: &str) -> Result<String, Box<dyn Error>> {
    //TODO make this async
    let file = File::open("data.json").unwrap();
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader).unwrap();

    let list_answers = config.answers.get(list_id).ok_or("List not found")?;

    Ok(format!(
        r#"<form action="{list_id}/send", method="post">
       {forms}
       </form>"#,
        list_id = list_id,
        forms = config
            .theses
            .iter()
            .map(|(id, thesis)| {
                //TODO fix panic
                as_form(&id, &thesis, &list_answers[id.as_str()])
            })
            .collect::<Vec<String>>()
            .join("\n")
    ))
}
