use std::collections::HashMap;

use rusqlite::{Connection, Statement, params, MappedRows, Row, Params};

use rocket::{Rocket, Build, State, tokio};
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;

use crate::util;
use std::sync::{Mutex, Arc};
use rocket::data::ToByteUnit;
use std::borrow::Borrow;
use rand::Rng;
use std::error::Error;
use webhook::client::{WebhookClient, WebhookResult};

// --- DB MANAGEMENT ---
pub struct BackendState {
    pub db: Mutex<Connection>
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
pub struct Article {
    pub id: String,
    pub title: String,
    pub author: String,
    pub date: u64,
    pub paper: String,
    pub issue: u64,
    pub image: String,
    pub style: u8,
    pub column: u8,
    pub sortnum: i16,
    pub content: String,
}

impl Article {
    pub fn get_context<'a>(self, state: &State<BackendState>) -> HashMap<&'a str, String> {
        let mut ctx: HashMap<&str, String> = HashMap::new();
        let paper = get_paper(state, &*self.paper);
        ctx.insert("title", self.title);
        ctx.insert("author", self.author);
        ctx.insert("date", self.date.to_string());
        ctx.insert("paper", paper.as_ref().map(|p| p.name.to_owned()).or_else(|| Some("UNKNOWN".to_string())).unwrap());
        ctx.insert("paper_id", paper.as_ref().map(|p| p.id.to_owned()).or_else(|| Some("ERROR".to_string())).unwrap());
        ctx.insert("issue", self.issue.to_string());
        ctx.insert("image", self.image);
        ctx.insert("article_json", util::js_pre(&*self.content));
        ctx.insert("logo", paper.as_ref().map(|p| p.logo.to_owned()).or_else(|| Some("ERROR".to_string())).unwrap());
        ctx
    }

    pub fn get_prev_context<'a>(self) -> HashMap<&'a str, String> {
        let mut ctx: HashMap<&str, String> = HashMap::new();
        ctx.insert("id", self.id);
        ctx.insert("title", self.title);
        ctx.insert("author", self.author);
        ctx.insert("image", self.image);
        ctx.insert("style", self.style.to_string());
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
            style: 0,
            column: 0,
            sortnum: 0,
            content: "{\"ops\":[{\"insert\":\"INVALID\"}]}".to_string(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
pub struct Paper {
    pub id: String,
    pub name: String,
    pub featured_issue: u64,
    pub logo: String
}

pub fn gen_code() -> u32 {
    rand::thread_rng().gen_range(1024..u32::MAX)
}

pub fn verify_code(state: &State<BackendState>, code: u32) -> bool {
    let db = state.db.lock().unwrap();
    let mut st = db.prepare("SELECT * FROM credentials WHERE code = ?").unwrap();

    let results = st.query_map(params![code], |row| row.get::<usize, u32>(0)).unwrap();

    for _ in results {
        return true;
    }
    false
}

pub fn send_code(url: &str, code: u32) {
    let c_url: String = url.to_owned();
    let c_code: u32 = code.to_owned();

    tokio::task::spawn(async move {
        let client = WebhookClient::new(&*c_url);
        client.send(|m| m
            .username("Website")
            .embed(|e| e
                .title(&*format!("Publishing Code: `{}`", c_code.to_string())))).await
    });
}

pub fn update_code(state: &State<BackendState>) {
    let new_code: u32 = gen_code();
    let db = state.db.lock().unwrap();
    db.execute("UPDATE credentials SET code = ?;", params![new_code]).unwrap();

    let mut st = db.prepare("SELECT hook_url FROM credentials").unwrap();
    let url_res = st.query_map([], |row| row.get::<usize, String>(0)).unwrap();

    for url in url_res {
        send_code(&*url.unwrap(), new_code);
        break;
    }
}

pub fn row_to_article<E>(row: &Row) -> Result<Article, E> where E: Error {
    Ok(Article {
        id: row.get(0).unwrap(),
        title: row.get(1).unwrap(),
        author: row.get(2).unwrap(),
        date: row.get(3).unwrap(),
        paper: row.get(4).unwrap(),
        issue: row.get(5).unwrap(),
        image: row.get(6).unwrap(),
        style: row.get(7).unwrap(),
        column: row.get(8).unwrap(),
        sortnum: row.get(9).unwrap(),
        content: row.get(10).unwrap(),
    })
}

pub fn get_article(state: &State<BackendState>, article_id: &str) -> Option<Article> {
    let db = state.db.lock().unwrap();
    let mut st = db.prepare("SELECT * FROM articles WHERE id = :id").unwrap();

    let articles = st.query_map(&[(":id", article_id)], row_to_article).unwrap();
    for article in articles {
        return Some(article.ok()?);
    }
    None
}

pub fn get_articles<P>(state: &State<BackendState>, where_cond: &str, params: P) -> Vec<Article> where P: Params {
    let db = state.db.lock().unwrap();
    let mut st = db.prepare(&*format!("SELECT * FROM articles WHERE {}", where_cond)).unwrap();

    let articles = st.query_map(params, row_to_article).unwrap();
    let mut vec = Vec::new();
    for article in articles {
        vec.push(article.ok().unwrap());
    }
    vec
}

pub fn put_article(state: &State<BackendState>, article: &Article) {
    state.db.lock().unwrap().execute("INSERT OR REPLACE INTO articles VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
               (&article.id, &article.title, &article.author, &article.date, &article.paper,
                &article.issue, &article.image, &article.style, &article.column, &article.sortnum, &article.content)).unwrap();
}

pub fn get_paper_select_contexts(state: &State<BackendState>) -> Vec<HashMap<&str, String>> {
    let db = state.db.lock().unwrap();
    let mut st = db.prepare("SELECT id, name FROM papers").unwrap();

    let maps = st.query_map([], |row| {
        let mut map: HashMap<&str, String> = HashMap::new();
        map.insert("id", row.get::<usize, String>(0).unwrap());
        map.insert("name", row.get::<usize, String>(1).unwrap());
        Ok(map)
    }).unwrap();

    let mut vec: Vec<HashMap<&str, String>> = Vec::new();
    for map in maps {
        vec.push(map.unwrap());
    }
    vec
}

pub fn get_paper(state: &State<BackendState>, paper_id: &str) -> Option<Paper> {
    let db = state.db.lock().unwrap();
    let mut st = db.prepare("SELECT * FROM papers WHERE id = :id").unwrap();

    let papers = st.query_map(&[(":id", paper_id)], |row| {
        Ok(Paper {
            id: row.get(0).unwrap(),
            name: row.get(1).unwrap(),
            featured_issue: row.get(2).unwrap(),
            logo: row.get(3).unwrap()
        })
    }).unwrap();

    for paper in papers {
        return Some(paper.ok()?);
    }
    None
}

pub fn put_paper(state: &State<BackendState>, paper: &Paper) {
    state.db.lock().unwrap().execute("INSERT OR REPLACE INTO papers VALUES (?, ?, ?, ?)",
                                     (&paper.id, &paper.name, &paper.featured_issue, &paper.logo)).unwrap();
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
    paper: Paper,
    code: u32
}

#[post("/create_paper", format = "json", data = "<paper>")]
fn api_create_paper(state: &State<BackendState>, paper: Json<PotentialPaper>) -> String {
    if verify_code(state, paper.code) {
        put_paper(state, &paper.into_inner().paper);
        update_code(state);
        return "Newspaper Published!".to_string();
    }

    "Incorrect Code!".to_string()
}

#[post("/publish", format = "json", data = "<article>")]
fn api_publish(state: &State<BackendState>, article: Json<PotentialArticle>) -> String {
    if verify_code(state, article.code) {
        put_article(state, &article.into_inner().article);
        update_code(state);
        return "Article Published!".to_string();
    }

    "Incorrect Code!".to_string()
}

#[get("/article/<id>")]
fn api_get_article(state: &State<BackendState>, id: &str) -> Json<Article> {
    get_article(state, id).or_else(|| Some(Article::invalid())).map(|a| Json::from(a)).unwrap()
}

#[get("/resend_code")]
fn api_resend_code(state: &State<BackendState>) -> String {
    let db = state.db.lock().unwrap();
    let mut st = db.prepare("SELECT code, hook_url FROM credentials").unwrap();

    let code_res = st.query_map([], |row| Ok((row.get::<usize, u32>(0).unwrap(), row.get::<usize, String>(1).unwrap()))).unwrap();
    for code in code_res {
        let u_code = code.unwrap();
        send_code(&*u_code.1, u_code.0);
        return "Code Sent".to_string();
    }

    "Unexpected Error".to_string()
}

pub fn start_rocket(builder: Rocket<Build>) -> Rocket<Build> {
    let db = Connection::open("./news_site_data.db").unwrap();
    db.execute("CREATE TABLE IF NOT EXISTS articles (id TEXT PRIMARY KEY, title TEXT, author TEXT, date INTEGER, paper TEXT, issue INTEGER, image TEXT, style INTEGER, column INTEGER, sortnum INTEGER, article_json TEXT);", ()).unwrap();
    db.execute("CREATE TABLE IF NOT EXISTS papers (id TEXT PRIMARY KEY, name TEXT, featured_issue INTEGER, logo TEXT);", ()).unwrap();
    db.execute("CREATE TABLE IF NOT EXISTS credentials (id INTEGER PRIMARY KEY, code INTEGER, hook_url TEXT);", ()).unwrap();
    db.execute("INSERT OR IGNORE INTO credentials VALUES (0, ?, '');", params![gen_code()]).unwrap();

    builder.mount("/api", routes![api_resend_code, api_publish, api_get_article, api_create_paper])
        .manage(BackendState { db: Mutex::new(db) })
}
