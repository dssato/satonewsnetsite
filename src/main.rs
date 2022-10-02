mod site;
mod data;
mod util;

#[macro_use] extern crate rocket;
use rocket_dyn_templates::{Template};
use rocket::fs::{FileServer, relative};

#[launch]
fn start() -> _ {
    site::start_rocket(
        rocket::build()
            .mount("/", FileServer::from(relative!("static")))
            .attach(Template::custom(|_engines| {}))
    )
}
