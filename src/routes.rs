use rand::{thread_rng, Rng};
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::{json, Json, Value};
use rocket::State;
use std::sync::Arc;
use std::time::Instant;

// use colored::*;
use crate::database::{add_race_records, add_records};
use crate::models::{AnswerData, Core, GameQuestion, GameState, RankEntry};
pub type Session<'a> = rocket_session::Session<'a, GameState>;

fn get_random_index(length: usize) -> usize {
    let mut rng = thread_rng();
    rng.gen_range(0..length)
}

fn get_four_random_choice(length: usize) -> Vec<usize> {
    let mut rng = rand::thread_rng();
    let mut choices = Vec::new();
    let mut len = 0;
    while len < 4 {
        let random_index = rng.gen_range(0..length);
        if !choices.contains(&random_index) {
            choices.push(random_index);
            len += 1;
        }
    }
    choices
}

#[get("/question?<begin_flag>")]
pub async fn get_question<'r>(
    image_paths: &State<Arc<Vec<String>>>,
    session: Session<'r>,
    begin_flag: bool,
) -> Result<Json<GameQuestion>, Custom<Value>> {
    if begin_flag {
        session.tap(|game_state| {
            game_state.current_time_used = Instant::now();
            game_state.current_correct = 0;
        });
    }
    let game_state = session.tap(|game_state| game_state.clone());

    if game_state.username.is_none() {
        return Err(Custom(
            Status::Unauthorized,
            json!({"error": "need_username"}),
        ));
    }

    let length = image_paths.len();
    if length < 5 {
        return Err(Custom(
            Status::InternalServerError,
            json!({"error": "Not enough images in the directory"}),
        ));
    }

    let question_index = get_random_index(length);
    let question_image_url = image_paths[question_index].clone();
    let previous_answer = &game_state.current_question.question_image_url;

    let mut choices = get_four_random_choice(length);
    if !game_state.previous_answer.is_none() {
        let previous_answer_index = image_paths
            .iter()
            .position(|x| x == previous_answer)
            .unwrap();

        choices[get_random_index(4)] = question_index;
        if !choices.contains(&previous_answer_index) {
            choices[get_random_index(4)] = previous_answer_index;
        }
    }

    let options: Vec<String> = choices
        .iter()
        .map(|&idx| image_paths[idx].to_string())
        .collect();

    let question = GameQuestion {
        id: 1,
        question_image_url: question_image_url.clone(),
        options,
    };

    if game_state.previous_answer.is_none() {
        session.tap(|game_state| {
            game_state.current_question = question.clone();
            game_state.previous_answer = Some(question.question_image_url.clone());
        });

        return Ok(Json(question));
    }

    session.tap(|game_state| {
        game_state.previous_answer = Some(previous_answer.clone());
        game_state.current_question = question.clone();
    });

    Ok(Json(question))
}

#[post("/answer?<is_race>", data = "<answer>")]
pub async fn submit_answer(
    answer: Json<AnswerData>,
    is_race: bool,
    session: Session<'_>,
    core: rocket_db_pools::Connection<Core>,
) -> Result<Json<Value>, Custom<Value>> {
    let game_state = session.tap(|gamestate| gamestate.clone());

    if game_state.username.is_none() {
        return Err(Custom(
            Status::Unauthorized,
            json!({"error": "need_username"}),
        ));
    }

    let answer_index = answer.0.answer;
    let correct_answer = game_state.previous_answer.as_ref().ok_or_else(|| {
        Custom(
            Status::InternalServerError,
            json!({"error": "Previous answer not found"}),
        )
    })?;

    if game_state.current_question.options.get(answer_index) == Some(correct_answer) {
        if session.tap(|gamestate| {
            gamestate.current_correct += 1;
            is_race && gamestate.current_correct == 30
        }) {
            let (time_used, correct_all, username) = session.tap(|gamestate| {
                gamestate.previous_answer = None;
                let current_correct = gamestate.current_correct;
                gamestate.current_correct = 0;
                (
                    gamestate.current_time_used.elapsed().as_millis(),
                    current_correct,
                    game_state.username,
                )
            });

            let _result = add_race_records(
                core,
                time_used as u32,
                correct_all as u32,
                username.expect("no username set"),
            )
            .await
            .expect("Fail to add record to database.");

            return Ok(Json(json!({
                "success": true,
                "upto": true,
                "time_used": time_used,
                "correct_all": correct_all
            })));
        };
        Ok(Json(json!({"success": true})))
    } else {
        let (time_used, correct_all, username) = session.tap(|gamestate| {
            gamestate.previous_answer = None;
            let current_correct = gamestate.current_correct;
            gamestate.current_correct = 0;
            (
                gamestate.current_time_used.elapsed().as_millis(),
                current_correct,
                game_state.username,
            )
        });

        if !is_race {
            let _result = add_records(
                core,
                time_used as u32,
                correct_all as u32,
                username.expect("no username set"),
            )
            .await
            .expect("Fail to add record to database.");
        };

        Ok(Json(json!({
            "success": false,
            "time_used": time_used,
            "correct_all": correct_all
        })))
    }
}

#[post("/set_username", data = "<username>")]
pub async fn set_username(
    username: Json<String>,
    session: Session<'_>,
) -> Result<Json<Value>, Custom<Value>> {
    let username = username.0;

    if username.is_empty() {
        return Err(Custom(
            Status::BadRequest,
            json!({"error": "Username cannot be empty"}),
        ));
    }

    session.tap(|game_state| {
        game_state.username = Some(username);
    });

    Ok(Json(json!({"message": "Username set successfully"})))
}

#[get("/leaderboard")]
pub async fn get_leaderboard(
    mut core: rocket_db_pools::Connection<Core>,
) -> Result<Json<Vec<RankEntry>>, Custom<Value>> {
    let query = r#"
        SELECT r.user_name, r.correct_num, r.used_time
        FROM rank r
        JOIN (
            SELECT user_name, MAX(correct_num) AS max_correct_num
            FROM rank
            GROUP BY user_name
        ) max_r ON r.user_name = max_r.user_name AND r.correct_num = max_r.max_correct_num
        ORDER BY r.correct_num DESC, r.used_time ASC
        LIMIT 100;
    "#;

    let entries: Vec<RankEntry> = sqlx::query_as(&query)
        .fetch_all(&mut **core)
        .await
        .map_err(|e| Custom(Status::InternalServerError, json!({"error": e.to_string()})))?;

    Ok(Json(entries))
}

#[get("/race-leaderboard")]
pub async fn get_race_leaderboard(
    mut core: rocket_db_pools::Connection<Core>,
) -> Result<Json<Vec<RankEntry>>, Custom<Value>> {
    let query = r#"
        SELECT user_name, correct_num, used_time
        FROM race_rank
        WHERE (user_name, used_time) IN (
            SELECT user_name, MIN(used_time)
            FROM race_rank
            GROUP BY user_name
        )
        ORDER BY used_time ASC
        LIMIT 100;
    "#;

    let entries: Vec<RankEntry> = sqlx::query_as(&query)
        .fetch_all(&mut **core)
        .await
        .map_err(|e| Custom(Status::InternalServerError, json!({"error": e.to_string()})))?;

    Ok(Json(entries))
}
