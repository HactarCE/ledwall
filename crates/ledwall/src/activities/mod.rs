use crate::Activity;

pub mod rainbow;
pub mod tetris;

pub fn init_activities() -> Vec<Box<dyn Activity>> {
    vec![
        Box::new(tetris::Tetris::default()),
        Box::new(rainbow::Rainbow::new()),
    ]
}
