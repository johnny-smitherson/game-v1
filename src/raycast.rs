//! This example will show you how to use your mouse cursor as a ray casting source, cast into the
//! scene, intersect a mesh, and mark the intersection with the built in debug cursor. If you are
//! looking for a more fully-featured mouse picking plugin, try out bevy_mod_picking.

use bevy::prelude::*;
use bevy_inspector_egui::InspectorOptions;
use bevy_mod_raycast::{prelude::*, IntersectionData};

pub struct RaycastPlugin;
impl Plugin for RaycastPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainRaycastResult>()
            .register_type::<TerrainRaycastResult>()
            // The DefaultRaycastingPlugin bundles all the functionality you might need into a single
            // plugin. This includes building rays, casting them, and placing a debug cursor at the
            // intersection. For more advanced uses, you can compose the systems in this plugin however
            // you need. For example, you might exclude the debug cursor system.
            .add_plugins(DefaultRaycastingPlugin::<TerrainRaycastSet>::default())
            // You will need to pay attention to what order you add systems! Putting them in the wrong
            // order can result in multiple frames of latency. Ray casting should probably happen near
            // start of the frame. For example, we want to be sure this system runs before we construct
            // any rays, hence the ".before(...)". You can use these provided RaycastSystem labels to
            // order your systems with the ones provided by the raycasting plugin.
            .add_systems(
                First,
                update_raycast_with_cursor.before(RaycastSystem::BuildRays::<TerrainRaycastSet>),
            )
            // .add_systems(Startup, setup)
            .add_systems(PreUpdate, update_terrain_raycast_result);
    }
}

/// This is a unit struct we will use to mark our generic `RaycastMesh`s and `RaycastSource` as part
/// of the same group, or "RaycastSet". For more complex use cases, you might use this to associate
/// some meshes with one ray casting source, and other meshes with a different ray casting source."
#[derive(Reflect)]
pub struct TerrainRaycastSet;

#[derive(Reflect, Resource, Debug, Default, InspectorOptions)]
#[reflect(Resource)]
pub struct TerrainRaycastResult {
    pub intersection: Option<IntersectionData>,
    pub caster_entity: Option<Entity>,
    pub hit_entity: Option<Entity>,
}

// Update our `RaycastSource` with the current cursor position every frame.
fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut RaycastSource<TerrainRaycastSet>>,
) {
    // Grab the most recent cursor event if it exists:
    let Some(cursor_moved) = cursor.iter().last() else {
        return;
    };
    for mut pick_source in &mut query {
        pick_source.cast_method = RaycastMethod::Screenspace(cursor_moved.position);
        pick_source.should_early_exit = true;
    }
}

fn update_terrain_raycast_result(
    query: Query<(Entity, &RaycastMesh<TerrainRaycastSet>)>,
    mut result: ResMut<TerrainRaycastResult>,
) {
    result.intersection = None;
    result.caster_entity = None;
    result.hit_entity = None;
    for (hit_entity, intersection) in query.iter() {
        for (caster_entity, intersection) in intersection.intersections.iter() {
            result.intersection = Some(intersection.clone());
            result.caster_entity = Some(*caster_entity);
            result.hit_entity = Some(hit_entity);
        }
    }
}

// // Set up a simple 3D scene
// fn setup(mut commands: Commands) {
//     // Overwrite the default plugin state with one that enables the debug cursor. This line can be
//     // removed if the debug cursor isn't needed as the state is set to default values when the
//     // default plugin is added.
//     commands
//         .insert_resource(RaycastPluginState::<TerrainRaycastSet>::default().with_debug_cursor());
// }
