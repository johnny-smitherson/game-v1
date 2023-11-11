use bevy::prelude::*;

use crate::menu::{egui_ui_system, mouse_is_over_menu, UiMarkHoverBundle, UiMarkMouseOverMenu};

use super::events::{TankCommandEvent, TankCommandEventType};
use super::tank::{PlayerControlledTank, Tank};

pub struct TankUiPlugin;
impl Plugin for TankUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tank_control_ui)
            .add_systems(
                PreUpdate,
                (send_events_button_press
                    .after(egui_ui_system)
                    .run_if(mouse_is_over_menu),)
                    .chain(),
            )
            .add_systems(PostUpdate, (update_labels,));
    }
}

pub fn spawn_tank_control_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    build_tank_control_ui(&mut commands, &asset_server);
}

#[derive(Component)]
enum TankUIButton {
    PowerPlus,
    PowerMinus,
    ElevationPlus,
    ElevationMinus,
    BearingPlus,
    BearingMinus,
    Fire,
}

#[allow(clippy::type_complexity)]
fn send_events_button_press(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BorderColor,
            &TankUIButton,
            Changed<Interaction>,
        ),
        (With<Button>, With<TankUIButton>),
    >,
    mut event_writer: EventWriter<TankCommandEvent>,
    tank_query: Query<Entity, With<PlayerControlledTank>>,
) {
    if let Ok(tank_entity) = tank_query.get_single() {
        for (interaction, mut border_color, button_tag, changed) in &mut interaction_query {
            match *interaction {
                Interaction::Pressed => {
                    let event_type = match button_tag {
                        TankUIButton::ElevationMinus => TankCommandEventType::ElevationMinus,
                        TankUIButton::ElevationPlus => TankCommandEventType::ElevationPlus,
                        TankUIButton::PowerPlus => TankCommandEventType::PowerPlus,
                        TankUIButton::PowerMinus => TankCommandEventType::PowerMinus,
                        TankUIButton::BearingMinus => TankCommandEventType::BearingLeft,
                        TankUIButton::BearingPlus => TankCommandEventType::BearingRight,
                        TankUIButton::Fire => TankCommandEventType::Fire,
                    };
                    // send Fire only when changed; send others every frame
                    if event_type != TankCommandEventType::Fire || changed {
                        event_writer.send(TankCommandEvent {
                            event_type,
                            tank_entity,
                        });
                    }

                    border_color.0 = Color::RED;
                }
                Interaction::Hovered => {
                    border_color.0 = Color::WHITE;
                }
                Interaction::None => {
                    border_color.0 = Color::WHITE;
                }
            }
        }
    }
}

#[derive(Component)]
enum TankUILabel {
    PowerLevel,
    Bearing,
    Elevation,
}

fn update_labels(
    mut query: Query<(&mut Text, &TankUILabel), With<TankUILabel>>,
    tank: Query<&Tank, With<PlayerControlledTank>>,
) {
    if let Ok(tank) = tank.get_single() {
        for (mut text, _type) in &mut query {
            match _type {
                TankUILabel::Bearing => {
                    text.sections[0].value = format!("{}", tank.bearing.to_degrees().round())
                }
                TankUILabel::Elevation => {
                    text.sections[0].value = format!("{}", tank.elevation.to_degrees().round())
                }
                TankUILabel::PowerLevel => {
                    text.sections[0].value = format!("{}", tank.power.round())
                }
            }
        }
    }
}

pub fn build_tank_control_ui(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
    let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");
    let root = build_tank_control_root(commands);
    build_tank_control_row(
        root,
        commands,
        &font,
        TankUIButton::PowerPlus,
        TankUIButton::PowerMinus,
        "Power",
        TankUILabel::PowerLevel,
    );
    build_tank_control_row(
        root,
        commands,
        &font,
        TankUIButton::ElevationPlus,
        TankUIButton::ElevationMinus,
        "Elevation",
        TankUILabel::Elevation,
    );
    build_tank_control_row(
        root,
        commands,
        &font,
        TankUIButton::BearingPlus,
        TankUIButton::BearingMinus,
        "Bearing",
        TankUILabel::Bearing,
    );

    // FIRE BUTTON
    let color = Color::rgba(0.9, 0.9, 0.9, 0.9);
    let button_bundle = ButtonBundle {
        style: Style {
            width: Val::Percent(95.0),
            height: Val::Percent(95.0),
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
        color,
    };

    commands.entity(root).with_children(|parent| {
        parent
            .spawn((
                // ROW
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(25.0),
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
                // === FIRE ===
                row.spawn((
                    button_bundle.clone(),
                    TankUIButton::Fire,
                    UiMarkMouseOverMenu,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("FIRE", text_style.clone()));
                });
            });
    });

    root
}

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
        color,
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
                            height: Val::Percent(25.0),
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
                    UiMarkHoverBundle::default(),
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
                    // === Minus Button ===
                    row.spawn((button_style.clone(), comp_minus))
                        .insert(UiMarkMouseOverMenu)
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("-", text_style.clone()));
                        });
                    // === Plus Button ===
                    row.spawn((button_style.clone(), comp_plus))
                        .insert(UiMarkMouseOverMenu)
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("+", text_style.clone()));
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
            UiMarkHoverBundle::default(),
        ))
        .id()
}
