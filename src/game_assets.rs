// use std::collections::{vec_deque, VecDeque};

use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_hanabi::prelude::*;
use bevy_inspector_egui::prelude::InspectorOptions;
use bevy_rapier3d::prelude::*;

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource)]
pub struct BulletAssets {
    pub flying_effect: Handle<EffectAsset>,
    pub hit_effect: Handle<EffectAsset>,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    #[reflect(ignore)]
    pub collider: Collider,
}

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource)]
pub struct GameSceneAssets {
    pub scenes: HashMap<String, Handle<Scene>>,
}

fn load_glb_scenes(mut scene_assets: ResMut<GameSceneAssets>, ass: Res<AssetServer>) {
    let filenames = [
        "Tanks and Armored Vehicle.glb",
        "Tanks and Armored Vehicle(1).glb",
        "Tanks and Armored Vehicle(2).glb",
        "Tanks and Armored Vehicle(3).glb",
    ];

    for prefix in ["ORIGINAL", "ANGLE_DISSOLVE"] {
        for filename in filenames {
            let mut path = "3d".to_owned();
            path.push('/');
            path.push_str(prefix);
            path.push('/');
            path.push_str(filename);
            let key = path.clone();
            path.push_str("#Scene0");
            let path = path;
            info!("LOADING GLB SCENE: {} into key 'key' {}", path, key);

            let my_gltf: Handle<Scene> = ass.load(path);

            scene_assets.scenes.insert(key, my_gltf);
        }
    }
}

pub struct GameAssetsPlugin;
impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
            .init_resource::<BulletAssets>()
            .register_type::<BulletAssets>()
            .init_resource::<GameSceneAssets>()
            .register_type::<GameSceneAssets>()
            .add_systems(PreStartup, (setup_bullet_assets, load_glb_scenes));
    }
}

fn setup_bullet_assets(
    mut effects: ResMut<Assets<EffectAsset>>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut bullet_assets: ResMut<BulletAssets>,
) {
    const BULLET_SIZE: f32 = 1.0;

    let mesh = meshes.add(Mesh::from(shape::Cube { size: BULLET_SIZE }));
    bullet_assets.mesh = mesh;
    let material = materials.add(Color::rgb(0.8, 0.7, 0.6).into());
    bullet_assets.material = material;
    bullet_assets.collider = Collider::cuboid(
        BULLET_SIZE / 2.0_f32,
        BULLET_SIZE / 2.0_f32,
        BULLET_SIZE / 2.0_f32,
    );

    bullet_assets.flying_effect = effects.add(get_portal_effect());
    bullet_assets.hit_effect = effects.add(get_firework_effect());
}

fn get_firework_effect() -> EffectAsset {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(0.1, Vec4::new(4.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(0.9, Vec4::new(4.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(4.0, 0.0, 0.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(0.9));
    size_gradient1.add_key(0.3, Vec2::splat(1.5));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let writer = ExprWriter::new();

    // Give a bit of variation by randomizing the age per particle. This will
    // control the starting color and starting size of particles.
    let age = writer.lit(0.).uniform(writer.lit(0.5)).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    // Give a bit of variation by randomizing the lifetime per particle
    let lifetime = writer.lit(1.8).uniform(writer.lit(2.5)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Add constant downward acceleration to simulate gravity
    let accel = writer.lit(Vec3::Y * -8.).expr();
    let update_accel = AccelModifier::new(accel);

    // Add drag to make particles slow down a bit after the initial explosion
    let drag = writer.lit(5.).expr();
    let update_drag = LinearDragModifier::new(drag);

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(2.).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Give a bit of variation by randomizing the initial speed
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: (writer.rand(ScalarType::Float) * writer.lit(70.) + writer.lit(60.)).expr(),
    };

    EffectAsset::new(32768, Spawner::once(666.0.into(), true), writer.finish())
        .with_name("firework")
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .update(update_drag)
        .update(update_accel)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient1,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient1,
            screen_space_size: false,
        })
        .render(BillboardModifier {})
}

fn get_portal_effect() -> EffectAsset {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(0.1, Vec4::new(4.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(0.9, Vec4::new(4.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(4.0, 0.0, 0.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(0.0));
    size_gradient1.add_key(0.2, Vec2::splat(1.5));
    size_gradient1.add_key(0.8, Vec2::splat(0.7));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let writer = ExprWriter::new();

    let ax_x = writer.lit(-1.5).uniform(writer.lit(1.5));
    let ax_y = writer.lit(-1.5).uniform(writer.lit(1.5));
    let ax_z = writer.lit(-1.5).uniform(writer.lit(1.5));
    let ax = writer.lit(Vec3::X) * ax_x + writer.lit(Vec3::Y) * ax_y + writer.lit(Vec3::Z) * ax_z;

    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: ax.expr(),
        radius: writer.lit(0.4).expr(),
        dimension: ShapeDimension::Volume,
    };

    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    // Give a bit of variation by randomizing the lifetime per particle
    let lifetime = writer.lit(0.5).uniform(writer.lit(0.8)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Add drag to make particles slow down a bit after the initial acceleration
    let drag = writer.lit(1.5).expr();
    let update_drag = LinearDragModifier::new(drag);

    let mut module = writer.finish();

    let tangent_accel = TangentAccelModifier::constant(&mut module, Vec3::ZERO, Vec3::Y, -13.);

    EffectAsset::new(32768, Spawner::rate(60.0.into()), module)
        .with_name("portal")
        .init(init_pos)
        .init(init_age)
        .init(init_lifetime)
        .update(update_drag)
        .update(tangent_accel)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient1,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient1,
            screen_space_size: false,
        })
        .render(BillboardModifier {})
}
