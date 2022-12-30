use backgammon_cli::backgammon::Game;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    let mut game = Game::new();
    game.start();
}
