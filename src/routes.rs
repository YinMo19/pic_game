use rand::{thread_rng, Rng};
use rocket::serde::json::{json, Json, Value};
use rocket::State;
use std::sync::Arc;
use std::time::Instant;

// use colored::*;

use crate::models::{AnswerData, GameQuestion, GameState};
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
) -> Result<Json<GameQuestion>, String> {
    if begin_flag {
        session.tap(|game_state| {
            game_state.current_time_used = Instant::now();
        });
    }
    let game_state = session.tap(|game_state| game_state.clone());

    let length = image_paths.len();
    if length < 5 {
        return Err("Not enough images in the directory".to_string());
    }

    let question_index = get_random_index(length);
    let question_image_url = image_paths[question_index].clone();
    let previous_answer = &game_state.current_question.question_image_url;

    let mut choices = get_four_random_choice(length);
    if !&game_state.previous_answer.is_none() {
        let previous_answer_index = image_paths
            .iter()
            .position(|x| x == previous_answer)
            .unwrap();
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

#[post("/answer", data = "<answer>")]
pub async fn submit_answer(
    answer: Json<AnswerData>,
    session: Session<'_>,
) -> Result<Value, String> {
    let game_state = session.tap(|gamestate| gamestate.clone());

    let answer_index = answer.0.answer;
    let correct_answer = game_state
        .previous_answer
        .as_ref()
        .ok_or_else(|| "Previous answer not found".to_string())?;

    if game_state.current_question.options.get(answer_index) == Some(correct_answer) {
        session.tap(|gamestate| {
            gamestate.current_correct += 1;
        });
        Ok(json!({"success": true}))
    } else {
        let (time_used, correct_all) = session.tap(|gamestate| {
            gamestate.previous_answer = None;
            let current_correct = gamestate.current_correct;
            gamestate.current_correct = 0;
            (
                gamestate.current_time_used.elapsed().as_millis(),
                current_correct,
            )
        });
        Ok(json!({
            "success": false,
            "time_used": time_used,
            "correct_all": correct_all
        }))
    }
}
