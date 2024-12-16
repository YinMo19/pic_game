use rocket::serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct GameQuestion {
    pub id: i32,
    pub question_image_url: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub previous_answer: Option<String>,
    pub current_question: GameQuestion,
    pub current_time_used: Instant,
    pub current_correct: u64,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            previous_answer: None,
            current_question: GameQuestion {
                id: 0,
                question_image_url: String::new(),
                options: vec![],
            },
            current_time_used: Instant::now(),
            current_correct: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct AnswerData {
    pub answer: usize,
}
