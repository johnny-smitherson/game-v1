use bevy::log::warn;
use game::create_game_app;

fn main() {
    create_game_app(false).run();

    warn!("game exiting");
}
