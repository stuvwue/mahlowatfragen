use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let file = File::open("questions.JSON").unwrap();
    let reader = BufReader::new(file);
    let questions: i::Questions = serde_json::from_reader(reader).unwrap();

    let file = File::open("parties.JSON").unwrap();
    let reader = BufReader::new(file);
    let parties: i::Parties = serde_json::from_reader(reader).unwrap();

    let mut lists = HashMap::new();
    for (key, val) in parties {
        //let index: u32 = key.chars[3].as_str().to_string();
        lists.insert(
            key.to_string(),
            o::List {
                name: val.longname.clone(),
                name_x: val.shortname.clone(),
            },
        );
    }

    let mut theses = HashMap::new();
    let mut i = 0;
    for question in questions.iter() {
        theses.insert(
            i.to_string(),
            o::Thesis {
                l: question.question.to_string(),
                s: question.title.to_string(),
                x: question.annotation.to_string(),
            },
        );
        i = i + 1;
    }
    let mut answers = HashMap::new();
    for key in lists.keys() {
        i = 0;
        answers.insert(key.to_string(), HashMap::new());
        let th = answers.get_mut(key).unwrap();
        for question in questions.iter() {
            let selection = match question.answers[key] {
                0 => "a",
                1 => "b",
                2 => "c",
                _ => panic!(),
            }
            .to_string();
            th.insert(
                i.to_string(),
                o::Answer {
                    selection,
                    statement: question.comments[key].to_string(),
                },
            );
            i = i + 1;
        }
    }
    let config = o::Config {
        lists,
        theses,
        answers,
    };

    println!("{}", serde_json::to_string_pretty(&config).unwrap());
}

mod i {
    use serde::Deserialize;
    use std::collections::HashMap;

    #[derive(Deserialize)]
    pub struct Party {
        pub shortname: String,
        pub longname: String,
        pub logo: String,
    }

    pub type Parties = HashMap<String, Party>;

    #[derive(Deserialize)]
    pub struct Question {
        pub title: String,
        pub question: String,
        pub annotation: String,
        pub style: String,
        pub answers: HashMap<String, u32>,
        pub comments: HashMap<String, String>,
    }

    pub type Questions = Vec<Question>;
}

mod o {
    use serde::Serialize;
    use std::collections::HashMap;

    #[derive(Serialize)]
    pub struct Thesis {
        pub l: String, //Question
        pub s: String, //Title
        pub x: String, //annotation
    }
    type Theses = HashMap<String, Thesis>;

    #[derive(Serialize)]
    pub struct List {
        pub name: String,   //Name
        pub name_x: String, //Short
    }
    type Lists = HashMap<String, List>;

    #[derive(Serialize)]
    pub struct Answer {
        pub selection: String,
        pub statement: String,
    }

    type Answers = HashMap<String, HashMap<String, Answer>>;

    #[derive(Serialize)]
    pub struct Config {
        pub theses: Theses,
        pub lists: Lists,
        pub answers: Answers,
    }
}
