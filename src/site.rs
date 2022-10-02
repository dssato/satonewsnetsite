use rocket::{Rocket, Build, State};
use rocket_dyn_templates::{Template, context};
use crate::data;

#[get("/read/<article>")]
fn article(state: &State<data::DBConfig>, article: String) -> Template {
    let article = data::get_article(state, &*article);
    if article.is_some() {
        return Template::render("article", article.unwrap().get_context())
    }
    Template::render("article", context! {
        title: "Article Not Found",
    })
}

#[get("/article")]
fn edit(state: &State<data::DBConfig>) -> Template {
    Template::render("edit_article", context! {

    })
}

#[get("/article/<article>")]
fn edit_article(state: &State<data::DBConfig>, article: String) -> Template {
    let article = data::get_article(state, &*article);
    if article.is_some() {
        return Template::render("article", article.unwrap().get_context())
    }
    Template::render("edit_article", context! {
        title: "Article Not Found",
    })
}

pub fn start_rocket(builder: Rocket<Build>) -> Rocket<Build> {
    data::start_rocket(builder.mount("/", routes![article])
        .mount("/edit", routes![edit, edit_article]))
}
