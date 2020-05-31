pub mod board;
pub mod rules;

fn main() {
    let b = board::new();
    board::draw(&b);
}
