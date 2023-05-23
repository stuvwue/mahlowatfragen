use clap::Parser;
use data::*;
use server::*;
use std::net::SocketAddr;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;

mod data;
mod server;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
enum Args {
    RunServer{socket: Option<SocketAddr>},
    GenTokens{
        base_url: String
    },
    ListTheses,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args {
        Args::RunServer{socket} => run_server(socket).await,
        Args::GenTokens{base_url} => gen_tokens(&base_url),
        Args::ListTheses => list_theses(),
    }
}

fn gen_tokens(url: &str) {
    let data = data::read_data().unwrap();
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

fn list_theses() {
    let data: Data = {
        let file = File::open("data.json").unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    };
    let mut theses_vec = data.theses.iter().collect::<Vec<_>>();
    theses_vec.sort_unstable_by_key(|kvp| kvp.0.parse::<u32>().unwrap_or(u32::MAX));
    for kvp in theses_vec {
        println!("{}: {}", kvp.0, kvp.1.s);
        println!("{}", kvp.1.l);
        println!("({})", kvp.1.x);
        println!();
    }
}
