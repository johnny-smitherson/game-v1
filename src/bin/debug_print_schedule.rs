use bevy::{
    asset::AssetEvents,
    ecs::schedule::BoxedScheduleLabel,
    prelude::*,
};
use game::create_game_app;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sched_str = &args[1];
    let out_path = &args[2];

    let sched: BoxedScheduleLabel = match sched_str.as_str() {
        "update" => Box::new(Update),
        "pre_update" => Box::new(PreUpdate),
        "post_update" => Box::new(PostUpdate),
        "first" => Box::new(First),
        "last" => Box::new(Last),
        "startup" => Box::new(Startup),
        "pre_startup" => Box::new(PreStartup),
        "post_startup" => Box::new(PostStartup),
        "asset_events" => Box::new(AssetEvents),
        _ => panic!("unknown sched type"),
    };
    let mut app = create_game_app(true);
    let settings = bevy_mod_debugdump::schedule_graph::Settings::default().filter_in_crate("game");

    // bevy_mod_debugdump::print_schedule_graph(&mut app, sched);
    let dot = bevy_mod_debugdump::schedule_graph_dot(&mut app, sched, &settings);
    std::fs::write(out_path, dot).expect("Unable to write file");
}
