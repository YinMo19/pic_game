#[macro_use]
extern crate rocket;
use rocket::fs::FileServer;
use std::sync::Arc;

mod routes;
use rocket_db_pools::Database;
use routes::{
    get_leaderboard, get_question, get_race_leaderboard, set_username, submit_answer, Session,
};

mod models;
use models::Core;
// use models::GameStates;
mod database;
use database::init_check_database_all;

use std::fs;
use std::path::Path;

fn init_pic() -> Vec<String> {
    let pic_dir = Path::new("./static/pic");
    let entries = fs::read_dir(pic_dir).expect("Failed to read directory");

    // collect all image paths
    entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.path().display().to_string()[8..].to_string())
        .filter(|path| path.ends_with(".jpg"))
        .collect()
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    init_check_database_all().await;
    let image_paths = Arc::new(init_pic());

    let _rocket = rocket::build()
        .attach(Core::init())
        .attach(Session::fairing())
        .manage(image_paths.clone())
        .mount("/", FileServer::from("./static"))
        .mount(
            "/",
            routes![
                get_question,
                submit_answer,
                set_username,
                get_leaderboard,
                get_race_leaderboard
            ],
        )
        .launch()
        .await?;

    Ok(())
}
