use backgammon_cli::backgammon::Game;
use std::env;

fn main() {
    if env::args().nth(1).as_deref() == Some("--debug") {
        unsafe {
            env::set_var("RUST_BACKTRACE", "1");
        }
    }

    let mut game = Game::new();
    game.start();
}
