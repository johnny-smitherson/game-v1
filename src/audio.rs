use bevy::{audio::PlaybackMode, prelude::*, utils::HashMap};
use bevy_inspector_egui::InspectorOptions;

use rand::seq::SliceRandom; // 0.7.2

/// NEGATIVE BECAUSE I DONT FUCKING KNOW
const SPATIAL_AUDIO_EAR_GAP: f32 = -0.15_f32;

pub struct GameAudioPlugin;
impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAudioAssets>()
            .register_type::<GameAudioAssets>()
            .add_event::<PlaySpatialAudioEvent>()
            .insert_resource(GlobalVolume::new(0.2))
            .add_systems(PreStartup, load_audio_folders)
            .add_systems(
                Last,
                (spawn_audio_bundles, update_spatial_audio_locations).chain(),
            );
    }
}

#[derive(Reflect, Event, Debug, Clone)]
pub struct PlaySpatialAudioEvent {
    /// the sound originates from this position; requires GlobalTransform
    parent_ent: Entity,
    /// key pointing to folder with audio assets
    asset_key: String,
    /// change effect every time, or keep same effect for same entity?
    randomize: bool,
    /// if make current item child of parent. BUG: after despawn, parent keeps invalid ref
    /// TODO: replace default despawn thing with one that removes the parent link before despawning
    attach_to_parent: bool,
    /// range 1 (only close hear) to 1000 (everyone hears)
    sound_reach: f32,
    /// relative around 1
    playback_speed: f32,
    /// relative around 1
    playback_volume: f32,
}

impl PlaySpatialAudioEvent {
    const ALL_KEYS: [&'static str; 3] = [
        "explosion/canon_fire",
        "explosion/distant_boom",
        "explosion/close_explosion",
    ];
    fn rand_speed() -> f32 {
        const SPEED_JITTER: f32 = 0.3;
        // get [0,1)
        let x: f32 = rand::random();
        // move to [-1,1)
        let x = x * 2.0 - 1.0;
        // move to [1-Jx, 1+Jx]
        1.0 + x * SPEED_JITTER
    }
    pub fn canon_fire(parent_ent: Entity) -> Self {
        Self {
            parent_ent,
            asset_key: "explosion/canon_fire".to_string(),
            randomize: false,
            attach_to_parent: false,
            sound_reach: 900.0,
            playback_speed: Self::rand_speed(),
            playback_volume: 0.3,
        }
    }
    pub fn distant_boom(parent_ent: Entity) -> Self {
        Self {
            parent_ent,
            asset_key: "explosion/distant_boom".to_string(),
            randomize: true,
            attach_to_parent: false,
            sound_reach: 1800.0,
            playback_speed: Self::rand_speed(),
            playback_volume: 0.6,
        }
    }
    pub fn close_explosion(parent_ent: Entity) -> Self {
        Self {
            parent_ent,
            asset_key: "explosion/close_explosion".to_string(),
            randomize: true,
            attach_to_parent: false,
            sound_reach: 100.0,
            playback_speed: Self::rand_speed(),
            playback_volume: 0.5,
        }
    }
}

#[derive(Reflect, Component, Default, InspectorOptions)]
pub struct SpatialAudioListener;

fn update_spatial_audio_locations(
    emitter_query: Query<(&GlobalTransform, &SpatialAudioSink, &SpatialAudioEffect)>,
    listener_query: Query<&GlobalTransform, With<SpatialAudioListener>>,
) {
    let Ok(listener) = listener_query.get_single() else {
        return;
    };
    let listener = listener.compute_transform();

    for (emitter, sink, effect_info) in &emitter_query {
        let emitter = emitter.compute_transform().translation;
        let emitter =
            listener.translation + (emitter - listener.translation) * effect_info.scale_range;
        sink.set_emitter_position(emitter);
        sink.set_listener_position(listener, SPATIAL_AUDIO_EAR_GAP);
    }
}

#[derive(Reflect, Component, Debug, Clone)]
struct SpatialAudioEffect {
    scale_range: f32,
}

fn spawn_audio_bundles(
    mut events: EventReader<PlaySpatialAudioEvent>,
    mut commands: Commands,
    audio_assets: Res<GameAudioAssets>,
    global_transform: Query<&GlobalTransform>,
    listener_query: Query<Entity, With<SpatialAudioListener>>,
) {
    for event in events.iter() {
        let source = if event.randomize {
            audio_assets.get_random(&event.asset_key)
        } else {
            audio_assets.get_for_entity(&event.asset_key, &event.parent_ent)
        };
        let listener_transform = if let Ok(listener_ent) = listener_query.get_single() {
            global_transform
                .get(listener_ent)
                .expect("listener has no transform")
                .compute_transform()
        } else {
            Transform::default()
        };
        let emitter_position = global_transform
            .get(event.parent_ent)
            .expect("parent has no transform")
            .compute_transform()
            .translation;
        let spatial = if event.attach_to_parent {
            SpatialBundle::default()
        } else {
            SpatialBundle::from_transform(Transform::from_translation(emitter_position))
        };
        let effect_ent = commands
            .spawn(spatial)
            .insert(SpatialAudioBundle {
                source,
                settings: PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    speed: event.playback_speed,
                    volume: bevy::audio::Volume::new_relative(event.playback_volume),
                    paused: false,
                },
                spatial: SpatialSettings::new(
                    listener_transform,
                    SPATIAL_AUDIO_EAR_GAP,
                    emitter_position,
                ),
            })
            .insert(Name::new("audio effect"))
            .insert(SpatialAudioEffect {
                scale_range: 1.0 / event.sound_reach,
            })
            .id();
        if event.attach_to_parent {
            commands.entity(effect_ent).set_parent(event.parent_ent);
        }
    }
}

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource)]
pub struct GameAudioAssets {
    pub folders: HashMap<String, Vec<Handle<AudioSource>>>,
}

fn entity_hash(ent: &Entity) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    ent.to_bits().hash(&mut hasher);
    hasher.finish()
}

impl GameAudioAssets {
    pub fn get_random(&self, key: &str) -> Handle<AudioSource> {
        let mut rng = rand::thread_rng();
        self.folders
            .get(key)
            .unwrap_or_else(|| panic!("KEY NOT FOUND: {key}"))
            .choose(&mut rng)
            .unwrap_or_else(|| panic!("KEY HAS NO ENTRIES: {key}"))
            .clone()
    }

    pub fn get_for_entity(&self, key: &str, ent: &Entity) -> Handle<AudioSource> {
        let vec = self
            .folders
            .get(key)
            .unwrap_or_else(|| panic!("KEY NOT FOUND: {key}"));
        vec[entity_hash(ent) as usize % vec.len()].clone()
    }
}

fn load_audio_folders(mut audio_assets: ResMut<GameAudioAssets>, ass: Res<AssetServer>) {
    for key in PlaySpatialAudioEvent::ALL_KEYS {
        let folder = format!("sound/ogg/{key}");
        if let Ok(handles) = ass.load_folder(folder) {
            let handles: Vec<Handle<AudioSource>> = handles
                .iter()
                .map(|x| x.clone().typed::<AudioSource>())
                .collect();
            audio_assets.folders.insert(key.to_string(), handles);
        }
    }
}
