use rocket::{Rocket, Build, State};
use rocket::local::blocking::Client;
use rocket_dyn_templates::{Template, context};

use crate::data;
use crate::util;
use rocket::response::Redirect;

use rusqlite::params;

#[get("/read/<article>")]
fn article(state: &State<data::BackendState>, article: String) -> Template {
    let res_article = data::get_article(state, &*article);
    if res_article.is_some() {
        return Template::render("article", res_article.unwrap().get_context(state));
    }
    Template::render("article", context! {
        title: "Article Not Found",
    })
}

#[get("/newspaper/<paper>")]
fn newspaper_name(state: &State<data::BackendState>, paper: String) -> Redirect {
    let res_paper = data::get_paper(state, &*paper);
    Redirect::found(format!("/newspaper/{}/{}", paper, res_paper.map(|p| p.featured_issue).or_else(|| Some(404)).unwrap()))
}

#[get("/newspaper/<paper>/<issue>")]
fn newspaper(state: &State<data::BackendState>, paper: String, issue: u64) -> Template {
    let res_paper = data::get_paper(state, &*paper);
    if res_paper.is_some() {
        let data = res_paper.unwrap();

        let mut headlines = data::get_articles(state, "paper = ? AND issue = ? AND column = 0", params![&*paper, issue]);
        headlines.sort_by(|a, b| a.sortnum.cmp(&b.sortnum));
        let mut left = data::get_articles(state, "paper = ? AND issue = ? AND column = 1", params![&*paper, issue]);
        left.sort_by(|a, b| a.sortnum.cmp(&b.sortnum));
        let mut right = data::get_articles(state, "paper = ? AND issue = ? AND column = 2", params![&*paper, issue]);
        right.sort_by(|a, b| a.sortnum.cmp(&b.sortnum));

        return Template::render("newspaper", context! {
            id: data.id,
            logo: data.logo,
            title: data.name,
            issue: issue,
            headlines: headlines.iter().map(|a| a.clone().get_prev_context()).collect::<Vec<_>>(),
            left: left.iter().map(|a| a.clone().get_prev_context()).collect::<Vec<_>>(),
            right: right.iter().map(|a| a.clone().get_prev_context()).collect::<Vec<_>>()
        });
    }
    Template::render("article", context! {
        title: "Newspaper Not Found",
    })
}

#[catch(404)]
fn page_not_found() -> Template {
    Template::render("article", context! {
        title: "404 (Wrong Link)",
    })
}

// ---- EDITING ----

#[get("/article")]
fn create_article(state: &State<data::BackendState>) -> Template {
    Template::render("edit", context! {
        article: context! {
            available_papers: data::get_paper_select_contexts(state)
        }
    })
}

#[get("/article/<article>")]
fn edit_article(state: &State<data::BackendState>, article: String) -> Template {
    let res_article = data::get_article(state, &*article);
    if res_article.is_some() {
        let data = res_article.unwrap();
        return Template::render("edit", context! {
            article: context! {
                id: data.id,
                title: data.title,
                author: data.author,
                paper: data.paper,
                issue: data.issue,
                image: data.image,
                style: data.style,
                column: data.column,
                sortnum: data.sortnum,
                article_json: util::js_pre(&*data.content),
                available_papers: data::get_paper_select_contexts(state)
            },
            edit: true
        });
    }
    Template::render("article", context! {
        title: "Article Not Found",
    })
}

#[get("/newspaper")]
fn create_newspaper() -> Template {
    Template::render("edit", context! {
        newspaper: context! { x: 0 }
    })
}

#[get("/newspaper/<paper>")]
fn edit_newspaper(state: &State<data::BackendState>, paper: String) -> Template {
    let res_paper = data::get_paper(state, &*paper);
    if res_paper.is_some() {
        let data = res_paper.unwrap();
        return Template::render("edit", context! {
            newspaper: context! {
                id: data.id,
                name: data.name,
                logo: data.logo,
                featured_issue: data.featured_issue
            },
            edit: true
        })
    }

    Template::render("article", context! {
        title: "Newspaper Not Found",
    })
}

// --- STATIC-ISH PAGES ---

#[get("/")]
fn home(state: &State<data::BackendState>) -> Template {
    Template::render("menu", context! {
        title: "Home",
        image: "/menu.png",
        import: "pages/home.html",
        links: data::get_papers(state).iter().map(|p| context! {
            url: format!("/newspaper/{}", p.id),
            name: &*p.name,
            info: format!("Latest Issue: No. {}", p.featured_issue),
            image: &*p.logo
        }).collect::<Vec<_>>()
    })
}

#[get("/submissions")]
fn submissions() -> Template {
    Template::render("menu", context! {
        title: "Submissions",
        image: "/menu.png",
        import: "pages/submissions.html",
        links: vec![
            context! {
                url: "https://forms.gle/99Ta9L5NVUcnVwGY6",
                name: "Submit Your News Here!",
                info: "Make sure you're on your school account",
                image: "https://growthsupermarket.com/wp-content/uploads/2018/06/GoogleForms_logo-e1529921391153.png"
            }
        ]
    })
}

#[get("/about")]
fn about() -> Template {
    Template::render("menu", context! {
        title: "Club Information",
        image: "/menu.png",
        import: "pages/about.html"
    })
}



pub fn start_rocket(builder: Rocket<Build>) -> Rocket<Build> {
    data::start_rocket(builder.register("/", catchers![page_not_found])
        .mount("/", routes![home, submissions, about, article, newspaper, newspaper_name])
        .mount("/edit", routes![create_article, edit_article, create_newspaper, edit_newspaper]))
}
