mod bullet;
mod bullet_physics;
mod events;
mod minimap;
mod tank;
mod tank_ai;
mod tank_kbd_shortcuts;
mod tank_ui;

use self::bullet::BulletPlugin;
use self::events::*;
use self::minimap::MinimapPlugin;
use self::tank::TankPlugin;
use self::tank_ai::TankAiPlugin;
use self::tank_kbd_shortcuts::KeyboardShortcutsPlugin;
use self::tank_ui::TankUiPlugin;
use bevy::prelude::*;

pub struct GameplayPlugin;
impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TankCommandEvent>()
            .add_event::<BulletHitEvent>()
            .add_plugins(TankPlugin)
            .add_plugins(BulletPlugin)
            .add_plugins(KeyboardShortcutsPlugin)
            .add_plugins(TankUiPlugin)
            .add_plugins(TankAiPlugin)
            .add_plugins(MinimapPlugin);
    }
}
