use game::create_game_app;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out_path = &args[1];

    let app = create_game_app(true);
    let settings = bevy_mod_debugdump::render_graph::Settings::default();
    let dot = bevy_mod_debugdump::render_graph_dot(&app, &settings);
    std::fs::write(out_path, dot).expect("Unable to write file");
}
