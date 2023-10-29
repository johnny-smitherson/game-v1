pub mod bullet;
pub mod tank;

use bevy::prelude::*;
use bullet::BulletPlugin;
use tank::TankPlugin;
pub struct GameplayPlugin;
impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TankPlugin).add_plugins(BulletPlugin);
    }
}
