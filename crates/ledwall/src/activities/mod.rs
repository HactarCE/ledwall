use crate::Activity;

pub mod life;
pub mod rainbow;
pub mod tetris;

pub fn init_activities() -> Vec<Box<dyn Activity>> {
    vec![
        Box::new(rainbow::Rainbow::default()),
        Box::new(tetris::Tetris::default()),
        Box::new(life::Life::default()),
    ]
}
