use std::error::Error;
use warp::{self, http::StatusCode, path, Filter, Rejection, Reply};

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::{
    env,
    sync::{Arc, Mutex},
};

#[tokio::main]
async fn main() {
    let mut args = env::args();
    args.next();
    let token_flag = args.next() == Some("gen-tokens".to_string());
    if token_flag {
        gen_tokens(&args.next().expect("expected a base url"));
    } else {
        run_server().await;
    }
}

const HTML_SAVED: &str = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>Antworten gespeichert</title>
  </head>
  <body>
    Ihre Antworten wurden gespeichert.<br>
    <a href="">Zurück zum Formular</a>
  </body>
</html>"#;

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

pub fn as_form(id: u32, thesis: &Thesis, answer: &Answer) -> String {
    format!(
        r#"<fieldset>
        <legend>Frage {id1}</legend>
        <h3> {title} </h3>
    {thesis}<br><br>
    <i>{hint}</i>
  <input type="radio" name="{id}selection" value="a" {approve}> Zustimmung
  <input type="radio" name="{id}selection" value="b" {neutral}> Neutral
  <input type="radio" name="{id}selection" value="c" {oppose}> Ablehnung
  <input type="radio" name="{id}selection" value="d" {skip}> Überspringen<br>
  <br>
  Begründung:<br>
  <textarea rows="5" cols="100" name="{id}statement">{statement}</textarea>
  </fieldset>"#,
        thesis = thesis.l.replace("\"", "&quot;"),
        title = thesis.s.replace("\"", "&quot;"),
        hint = if thesis.x != "" {
            format!("Hinweis: {}<br><br>", thesis.x.replace("\"", "&quot;"))
        } else {
            "".to_string()
        },
        id = id,
        id1 = id + 1,
        approve = if answer.selection == "a" {
            "checked"
        } else {
            ""
        },
        neutral = if answer.selection == "b" {
            "checked"
        } else {
            ""
        },
        oppose = if answer.selection == "c" {
            "checked"
        } else {
            ""
        },
        skip = if answer.selection == "d" {
            "checked"
        } else {
            ""
        },
        statement = answer.statement.replace("\"", "&quot;"),
    )
}

type DataM = Arc<Mutex<Data>>;

fn gen_tokens(url: &str) {
    let data: Data = {
        let file = File::open("data.json").unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    };
    let tokens = data
        .lists
        .into_iter()
        .map(|(id, list)| {
            (
                format!(
                    "{}{}",
                    list.name_x.split_whitespace().next().unwrap(),
                    random_token()
                ),
                id,
            )
        })
        .collect::<HashMap<_, _>>();

    let token_file = File::create("tokens.json").unwrap();
    serde_json::to_writer_pretty(token_file, &tokens).unwrap();

    let urls = tokens
        .iter()
        .map(|(token, _id)| format!("{}/{}", url, token))
        .collect::<Vec<String>>()
        .join("\n");
    fs::write("urls.txt", &urls).unwrap();
}

fn random_token() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(30).collect()
}

async fn run_server() {
    let data_raw: Data = {
        let file = File::open("data.json").unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    };
    let data = Arc::new(Mutex::new(data_raw));
    let token_raw: HashMap<String, String> = {
        let file = File::open("tokens.json").unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    };
    let tokens = Arc::new(token_raw);

    let with_data = move || {
        let data_clone = data.clone();
        warp::any().map(move || data_clone.clone())
    };
    let with_tokens = move || {
        let tokens_clone = tokens.clone();
        warp::any().map(move || tokens_clone.clone())
    };

    let extract_token = |token: String, token_map: TokenMap| {
        let result = match token_map.get(&token) {
            Some(id) => Ok(id.to_string()),
            None => Err(warp::reject()),
        };
        async { result }
    };

    type TokenMap = Arc<HashMap<String, String>>;

    let form = warp::get()
        .and(path!(String))
        .and(with_tokens())
        .and_then(extract_token)
        .and(with_data())
        .and_then(move |list_id: String, data: DataM| {
            let result = match read_and_format_forms(&list_id, data) {
                Ok(forms) => Ok(warp::reply::html(forms)),
                Err(_err) => Err(warp::reject()),
            };
            async { result }
        });

    let reply = warp::post()
        .and(path!(String))
        //.unify()
        .and(with_tokens())
        .and_then(extract_token)
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::form())
        .and(with_data())
        .map(|list_id, form: HashMap<String, String>, data_m: DataM| {
            let mut answers: HashMap<String, Answer> = HashMap::new();
            for (key, value) in form.into_iter() {
                let (id, field) = key.split_at(key.find('s').unwrap());
                let mut answer = answers.entry(id.to_string()).or_default();
                if field == "selection" {
                    (*answer).selection = value.to_string();
                } else if field == "statement" {
                    (*answer).statement = value.to_string();
                }
            }
            let mut data = data_m.lock().expect("failed to unlock mutex");
            data.answers.insert(list_id, answers);
            let file = File::create("data.json").unwrap();
            let writer = BufWriter::new(file);
            serde_json::to_writer_pretty(writer, &*data).unwrap();
            Ok(warp::reply::html(HTML_SAVED))
        });

    async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
        if err.is_not_found() {
            Ok(warp::reply::with_status(
                "Kandidat nicht gefunden",
                StatusCode::NOT_FOUND,
            ))
        } else {
            Err(err)
        }
    }

    warp::serve(reply.or(form.recover(handle_rejection)))
        .run(([0, 0, 0, 0], 8080))
        .await;
}

fn read_and_format_forms(list_id: &str, data_m: DataM) -> Result<String, Box<dyn Error>> {
    let data = data_m.lock().expect("failed to lock mutex");

    let list_answers = data.answers.get(list_id).ok_or("Candidate not found")?;

    let mut theses = data
        .theses
        .keys()
        .map(|id| id.parse::<u32>())
        .collect::<Result<Vec<u32>, _>>()?;
    theses.sort_unstable();
    Ok(format!(
        r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>Fragen zum Mahl-o-Wat</title>
  </head>
  <body>
    <form method="post">
      <input type="submit" value="Alle Eingaben speichern"><br>
        {forms}<br>
      <input type="submit" value="Alle Eingaben speichern"><br>
    </form>
  </body>
</html>"#,
        forms = theses
            .iter()
            .map(|id| {
                //TODO fix panic
                as_form(
                    *id,
                    &data.theses[&id.to_string()],
                    &list_answers[&id.to_string()],
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    ))
}
