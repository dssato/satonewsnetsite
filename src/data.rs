use std::collections::HashMap;

use rusqlite::{Connection, Statement, params};

use rocket::{Rocket, Build, State};
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;

use crate::util;
use std::sync::{Mutex, Arc};
use rocket::data::ToByteUnit;
use std::borrow::Borrow;

// --- DB MANAGEMENT ---
pub struct DBConfig {
    pub db: Mutex<Connection>
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
pub struct Article {
    id: String,
    title: String,
    author: String,
    date: u64,
    paper: String,
    issue: u64,
    image: String,
    content: String,
}

impl Article {
    pub fn get_context<'a>(self) -> HashMap<&'a str, String> {
        let mut ctx: HashMap<&str, String> = HashMap::new();
        ctx.insert("title", self.title);
        ctx.insert("author", self.author);
        ctx.insert("date", self.date.to_string());
        ctx.insert("paper", get_paper_name(&*self.paper));
        ctx.insert("issue", self.issue.to_string());
        ctx.insert("image", self.image);
        ctx.insert("article_json", util::js_pre(&*self.content));
        ctx
    }

    fn invalid() -> Article {
        Article {
            id: "INVALID".to_string(),
            title: "INVALID".to_string(),
            author: "INVALID".to_string(),
            date: 0,
            paper: "INVALID".to_string(),
            issue: 0,
            image: "INVALID".to_string(),
            content: "{\"ops\":[{\"insert\":\"INVALID\"}]}".to_string(),
        }
    }
}

pub fn get_article(state: &State<DBConfig>, article_id: &str) -> Option<Article> {
    let db = state.db.lock().unwrap();
    let mut st = db.prepare("SELECT * FROM articles WHERE id = :id").unwrap();

    let articles = st.query_map(&[(":id", article_id)], |row| {
        println!("GOT THIS {}", row.get::<usize, String>(0).unwrap());
        Ok(Article {
            id: row.get(0).unwrap(),
            title: row.get(1).unwrap(),
            author: row.get(2).unwrap(),
            date: row.get(3).unwrap(),
            paper: row.get(4).unwrap(),
            issue: row.get(5).unwrap(),
            image: row.get(6).unwrap(),
            content: row.get(7).unwrap(),
        })
    }).unwrap();

    for article in articles {
        return Some(article.ok()?);
    }
    None
}

pub fn put_article(state: &State<DBConfig>, article: &Article) {
    state.db.lock().unwrap().execute("INSERT OR REPLACE INTO articles VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
               (&article.id, &article.title, &article.author, &article.date, &article.paper, &article.issue, &article.image, &article.content)).unwrap();
}

pub fn put_paper(state: &State<DBConfig>, paper: &PotentialPaper) {
    state.db.lock().unwrap().execute("INSERT OR REPLACE INTO papers VALUES (?, ?)",
                                     (&paper.id, &paper.name)).unwrap();
}

pub fn get_paper_name(paper_id: &str) -> String {
    "".to_string()
}

// --- API ---

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct PotentialArticle {
    article: Article,
    code: u32
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct PotentialPaper {
    id: String,
    name: String
}

#[post("/create_paper", format = "json", data = "<article>")]
fn api_create_paper(state: &State<DBConfig>, article: Json<PotentialArticle>) -> String {
    // TODO: Check the code
    put_article(state, &article.into_inner().article);
    "Created".to_string()
}

#[post("/publish", format = "json", data = "<article>")]
fn api_publish(state: &State<DBConfig>, article: Json<PotentialArticle>) -> String {
    // TODO: Check the code
    put_article(state, &article.into_inner().article);
    "Published".to_string()
}

#[get("/article/<id>")]
fn api_get_article(state: &State<DBConfig>, id: &str) -> Json<Article> {
    get_article(state, id).or_else(|| Some(Article::invalid())).map(|a| Json::from(a)).unwrap()
}

pub fn start_rocket(builder: Rocket<Build>) -> Rocket<Build> {
    let db = Connection::open("./articles.db").unwrap();
    db.execute("CREATE TABLE IF NOT EXISTS articles (id TEXT PRIMARY KEY, title TEXT, author TEXT, date INTEGER, paper TEXT, issue INTEGER, image TEXT, article_json TEXT);", ()).unwrap();
    db.execute("CREATE TABLE IF NOT EXISTS papers (id TEXT PRIMARY KEY, name TEXT);", ()).unwrap();

    builder.mount("/api", routes![api_publish, api_get_article])
        .manage(DBConfig { db: Mutex::new(db) })
}
