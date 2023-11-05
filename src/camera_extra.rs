use bevy::{
    core_pipeline::{clear_color::ClearColorConfig, tonemapping::Tonemapping},
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
        // view::RenderLayers,
    },
};
use bevy_inspector_egui::InspectorOptions;

pub struct ExtraCameraPlugin;
impl Plugin for ExtraCameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ExtraCamera>();
    }
}

#[derive(Component, Reflect, InspectorOptions)]
pub struct ExtraCamera {
    render_target: Handle<Image>,
    rez: u32,
}

impl ExtraCamera {
    pub fn rez(&self) -> f32 {
        self.rez as f32
    }
    pub fn render_target_image_bundle(&self) -> ImageBundle {
        ImageBundle {
            image: UiImage::new(self.render_target.clone()),
            style: Style {
                // min_width: Val::Px(self.rez as f32),
                // min_height: Val::Px(self.rez as f32),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Default::default()
            },
            background_color: Color::WHITE.into(),
            ..Default::default()
        }
    }
}

pub fn spawn_extra_camera(
    commands: &mut Commands,
    images: &mut ResMut<Assets<Image>>,
    rez: u32,
    extra_comp: impl Bundle,
) {
    let size = Extent3d {
        width: rez,
        height: rez,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    image.resize(size);

    let image_handle = images.add(image);

    let camera = Camera3dBundle {
        camera_3d: Camera3d {
            clear_color: ClearColorConfig::Custom(Color::WHITE),
            ..default()
        },
        camera: Camera {
            // render before the "main pass" camera
            order: -1,
            target: RenderTarget::Image(image_handle.clone()),

            ..default()
        },
        projection: Projection::Perspective(PerspectiveProjection {
            near: 0.1,
            far: 100000.0,
            fov: (90.0 / 360.0) * (std::f32::consts::PI * 2.0),
            ..default()
        }),
        tonemapping: Tonemapping::BlenderFilmic,
        transform: Transform::from_translation(Vec3::new(0.0, 2000.0, 0.0))
            .looking_at(Vec3::ZERO, Vec3::Z),

        ..default()
    };

    let minimap = ExtraCamera {
        render_target: image_handle,
        rez,
    };

    commands
        .spawn(camera)
        .insert(minimap)
        .insert(extra_comp)
        .insert(RenderLayers::from_layers(&[0, 1]));
}
