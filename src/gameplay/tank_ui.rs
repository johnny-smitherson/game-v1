//! This example illustrates how to create a button that changes color and text based on its
//! interaction state.

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

pub struct TankUiPlugin;
impl Plugin for TankUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tank_control_ui);
        // .add_systems(Update, button_system);
        // .add_systems(Startup, setup_text)
        // .add_systems(Update, (text_update_system, text_color_system))
        // .add_systems(Startup, spawn_layout);
    }
}

pub fn spawn_tank_control_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    build_tank_control_ui(&mut commands, &asset_server);
}

#[derive(Component)]
struct TankUIRoot;

#[derive(Component)]
struct PowerPlusButton;

#[derive(Component)]
struct PowerMinusButton;

#[derive(Component)]
struct ElevationPlusButton;

#[derive(Component)]
struct ElevationMinusButton;

#[derive(Component)]
struct BearingPlusButton;

#[derive(Component)]
struct BearingMinusButton;

#[derive(Component)]
struct BearingLabel;

#[derive(Component)]
struct ElevationLabel;

#[derive(Component)]
struct PowerLabel;

pub fn build_tank_control_ui(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
    let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");
    let root = build_tank_control_root(commands);
    build_tank_control_row(
        root,
        commands,
        &font,
        PowerPlusButton,
        PowerMinusButton,
        "Power",
        PowerLabel,
    );
    build_tank_control_row(
        root,
        commands,
        &font,
        ElevationPlusButton,
        ElevationMinusButton,
        "Elevation",
        ElevationLabel,
    );
    build_tank_control_row(
        root,
        commands,
        &font,
        BearingPlusButton,
        BearingMinusButton,
        "Bearing",
        BearingLabel,
    );

    root
}

// fn make_container(parent: &mut ChildBuilder, width_percent: f32, height_percent: f32, flex_direction: FlexDirection, background_color: BackgroundColor) -> bevy::ecs::system::EntityCommands {
//     parent
//     .spawn((
//         NodeBundle {
//             style: Style {
//                 width: Val::Percent(width_percent),
//                 height: Val::Percent(height_percent),
//                 flex_direction,
//                 align_items: AlignItems::Center,
//                 justify_content: JustifyContent::Center,
//                 row_gap: Val::Px(5.0),
//                 column_gap: Val::Px(5.0),
//                 ..default()
//             },
//             background_color,
//             ..default()
//         },
//     ))
// }

fn build_tank_control_row(
    parent_id: Entity,
    commands: &mut Commands,
    font: &Handle<Font>,
    comp_plus: impl Component,
    comp_minus: impl Component,
    title: &str,
    comp_value_label: impl Component,
) -> Entity {
    let color = Color::rgba(0.9, 0.9, 0.9, 0.9);
    let button_style = ButtonBundle {
        style: Style {
            width: Val::Px(40.0),
            height: Val::Px(40.0),
            border: UiRect::all(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        border_color: BorderColor(color),
        background_color: Color::NONE.into(),
        ..default()
    };

    let text_style = TextStyle {
        font: font.clone(),
        font_size: 27.0,
        color: color,
    };

    commands
        .entity(parent_id)
        .with_children(|parent| {
            parent
                .spawn((
                    // ROW
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(33.0),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            row_gap: Val::Px(5.0),
                            column_gap: Val::Px(15.0),
                            ..default()
                        },
                        background_color: Color::Rgba {
                            red: 0.0,
                            blue: 0.0,
                            green: 0.0,
                            alpha: 0.1,
                        }
                        .into(),
                        ..default()
                    },
                    // Comp for whole row?,
                ))
                .with_children(|row| {
                    // === TITLE ===
                    row.spawn((NodeBundle {
                        style: Style {
                            width: Val::Percent(33.0),
                            height: Val::Percent(90.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Start,
                            ..default()
                        },
                        ..default()
                    },))
                        .with_children(|cell| {
                            cell.spawn(TextBundle::from_section(title, text_style.clone()));
                        });

                    // === Value ===
                    row.spawn((NodeBundle {
                        style: Style {
                            width: Val::Percent(33.0),
                            height: Val::Percent(90.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::End,
                            ..default()
                        },
                        ..default()
                    },))
                        .with_children(|cell| {
                            cell.spawn((
                                comp_value_label,
                                TextBundle::from_section("666", text_style.clone()),
                            ));
                        });
                    // === Plus Button ===
                    row.spawn((button_style.clone(), comp_plus))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("+", text_style.clone()));
                        });

                    // === Minus Button ===
                    row.spawn((button_style.clone(), comp_minus))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("-", text_style.clone()));
                        });
                });
        })
        .id()
}

fn build_tank_control_root(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(300.0),
                    height: Val::Px(150.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    position_type: PositionType::Absolute,
                    row_gap: Val::Px(5.0),
                    column_gap: Val::Px(5.0),
                    left: Val::VMin(3.0),
                    bottom: Val::VMin(3.0),
                    ..default()
                },
                ..default()
            },
            TankUIRoot,
        ))
        .id()
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn setup_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let font_size: f32 = 40.0;
    commands
        .spawn(NodeBundle {
            style: Style {
                height: Val::Percent(20.0),
                align_items: AlignItems::FlexEnd,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Button",
                        TextStyle {
                            font,
                            font_size,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                });
        });
}

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                text.sections[0].value = "Press".to_string();
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::RED;
            }
            Interaction::Hovered => {
                text.sections[0].value = "Hover".to_string();
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                text.sections[0].value = "Button".to_string();
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
                border_color.0 = Color::BLACK;
            }
        }
    }
}

// A unit struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
struct FpsText;

// A unit struct to help identify the color-changing Text component
#[derive(Component)]
struct ColorText;

fn setup_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Text with one section
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "hello\nbevy!",
            TextStyle {
                // This font is loaded and will be used instead of the default font.
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 100.0,
                color: Color::WHITE,
            },
        ) // Set the alignment of the Text
        .with_text_alignment(TextAlignment::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(15.0),
            ..default()
        }),
        ColorText,
    ));
    // Text with multiple sections
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: 60.0,
                color: Color::GOLD,
                // If no font is specified, it will use the default font.
                ..default()
            }),
        ]),
        FpsText,
    ));
}

fn text_color_system(time: Res<Time>, mut query: Query<&mut Text, With<ColorText>>) {
    for mut text in &mut query {
        let seconds = time.elapsed_seconds();

        // Update the color of the first and only section.
        text.sections[0].style.color = Color::Rgba {
            red: (1.25 * seconds).sin() / 2.0 + 0.5,
            green: (0.75 * seconds).sin() / 2.0 + 0.5,
            blue: (0.50 * seconds).sin() / 2.0 + 0.5,
            alpha: 1.0,
        };
    }
}

fn text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}

const ALIGN_ITEMS_COLOR: Color = Color::rgb(1., 0.066, 0.349);
const JUSTIFY_CONTENT_COLOR: Color = Color::rgb(0.102, 0.522, 1.);
const MARGIN: Val = Val::Px(5.);

fn spawn_layout(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands
        .spawn(NodeBundle {
            style: Style {
                // fill the entire window
                width: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color: BackgroundColor(Color::BLACK),
            ..Default::default()
        })
        .with_children(|builder| {
            // spawn the key
            builder
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        margin: UiRect::top(MARGIN),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|builder| {
                    spawn_nested_text_bundle(
                        builder,
                        font.clone(),
                        ALIGN_ITEMS_COLOR,
                        UiRect::right(MARGIN),
                        "AlignItems",
                    );
                    spawn_nested_text_bundle(
                        builder,
                        font.clone(),
                        JUSTIFY_CONTENT_COLOR,
                        UiRect::default(),
                        "JustifyContent",
                    );
                });

            builder
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(850.),
                        height: Val::Px(1020.),
                        flex_direction: FlexDirection::Column,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|builder| {
                    // spawn one child node for each combination of `AlignItems` and `JustifyContent`
                    let justifications = [
                        JustifyContent::FlexStart,
                        JustifyContent::Center,
                        JustifyContent::FlexEnd,
                        JustifyContent::SpaceEvenly,
                        JustifyContent::SpaceAround,
                        JustifyContent::SpaceBetween,
                    ];
                    let alignments = [
                        AlignItems::Baseline,
                        AlignItems::FlexStart,
                        AlignItems::Center,
                        AlignItems::FlexEnd,
                        AlignItems::Stretch,
                    ];
                    for justify_content in justifications {
                        builder
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .with_children(|builder| {
                                for align_items in alignments {
                                    spawn_child_node(
                                        builder,
                                        font.clone(),
                                        align_items,
                                        justify_content,
                                    );
                                }
                            });
                    }
                });
        });
}

fn spawn_child_node(
    builder: &mut ChildBuilder,
    font: Handle<Font>,
    align_items: AlignItems,
    justify_content: JustifyContent,
) {
    builder
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items,
                justify_content,
                width: Val::Px(160.),
                height: Val::Px(160.),
                margin: UiRect::all(MARGIN),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::DARK_GRAY),
            ..Default::default()
        })
        .with_children(|builder| {
            let labels = [
                (format!("{align_items:?}"), ALIGN_ITEMS_COLOR, 0.),
                (format!("{justify_content:?}"), JUSTIFY_CONTENT_COLOR, 3.),
            ];
            for (text, color, top_margin) in labels {
                // We nest the text within a parent node because margins and padding can't be directly applied to text nodes currently.
                spawn_nested_text_bundle(
                    builder,
                    font.clone(),
                    color,
                    UiRect::top(Val::Px(top_margin)),
                    &text,
                );
            }
        });
}

fn spawn_nested_text_bundle(
    builder: &mut ChildBuilder,
    font: Handle<Font>,
    background_color: Color,
    margin: UiRect,
    text: &str,
) {
    builder
        .spawn(NodeBundle {
            style: Style {
                margin,
                padding: UiRect {
                    top: Val::Px(1.),
                    left: Val::Px(5.),
                    right: Val::Px(5.),
                    bottom: Val::Px(1.),
                },
                ..Default::default()
            },
            background_color: BackgroundColor(background_color),
            ..Default::default()
        })
        .with_children(|builder| {
            builder.spawn(TextBundle::from_section(
                text,
                TextStyle {
                    font,
                    font_size: 24.0,
                    color: Color::BLACK,
                },
            ));
        });
}
