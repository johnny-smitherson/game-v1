pub mod bullet;
pub mod tank;
pub mod tank_control;
pub mod tank_ui;

use self::bullet::BulletPlugin;
use self::tank::TankPlugin;
use self::tank_control::KeyboardShortcutsPlugin;
use self::tank_ui::TankUiPlugin;
use bevy::prelude::*;

pub struct GameplayPlugin;
impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TankPlugin)
            .add_plugins(BulletPlugin)
            .add_plugins(KeyboardShortcutsPlugin)
            .add_plugins(TankUiPlugin);
    }
}
