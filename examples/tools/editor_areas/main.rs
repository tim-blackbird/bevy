#![allow(missing_docs)]

mod ui;

use bevy::{color::palettes::tailwind::*, prelude::*};

#[derive(Resource)]
struct BColors {
    void: Srgba,
    background: Srgba,
}

impl Default for BColors {
    fn default() -> Self {
        Self {
            void: Srgba::hex("242424").unwrap(),
            background: Srgba::hex("1c1c1c").unwrap(),
        }
    }
}

const SPACING: f32 = 5.;
const BORDER_RADIUS: f32 = 4.;

enum Mode {
    AreaA,
    AreaB,
    AreaC,
    AreaD,
    AreaE,
}

impl Mode {
    fn color(&self) -> Srgba {
        match self {
            Mode::AreaA => RED_500,
            Mode::AreaB => GREEN_500,
            Mode::AreaC => BLUE_500,
            Mode::AreaD => ORANGE_500,
            Mode::AreaE => PURPLE_500,
        }
    }
    fn name(&self) -> String {
        match self {
            Mode::AreaA => "Area A",
            Mode::AreaB => "Area B",
            Mode::AreaC => "Area C",
            Mode::AreaD => "Area D",
            Mode::AreaE => "Area E",
        }
        .to_string()
    }
}

enum SplitDirection {
    Horizontal,
    Vertical,
}

enum Child {
    Area {
        mode: Mode,
    },
    Split {
        direction: SplitDirection,
        children: Vec<(f32, Child)>,
    },
}

struct Workspace {
    name: String,
    child: Child,
}

#[derive(Resource)]
struct Workspaces(Vec<Workspace>);

fn main() {
    let mut app = App::new();

    app.insert_resource(Workspaces(vec![Workspace {
        name: "Default".to_string(),
        child: Child::Split {
            direction: SplitDirection::Horizontal,
            children: vec![
                (
                    0.15,
                    Child::Split {
                        direction: SplitDirection::Vertical,
                        children: vec![
                            (0.8, Child::Area { mode: Mode::AreaD }),
                            (0.2, Child::Area { mode: Mode::AreaE }),
                        ],
                    },
                ),
                (
                    0.7,
                    Child::Split {
                        direction: SplitDirection::Vertical,
                        children: vec![
                            (0.85, Child::Area { mode: Mode::AreaA }),
                            (0.15, Child::Area { mode: Mode::AreaC }),
                        ],
                    },
                ),
                (0.15, Child::Area { mode: Mode::AreaB }),
            ],
        },
    }]));

    app.init_resource::<BColors>();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Bevy Editor".to_string(),
            ..default()
        }),
        ..default()
    }))
    .add_systems(Startup, setup);
    app.run();
}

fn setup(e: Res<Workspaces>, colors: Res<BColors>, mut commands: Commands) {
    commands.spawn(Camera3dBundle { ..default() });

    let root = commands
        .spawn(NodeBundle {
            background_color: colors.void.into(),
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(SPACING)),
                ..default()
            },
            ..default()
        })
        .id();

    let workspaces_root = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        })
        .set_parent(root)
        .id();
    let areas_root = commands
        .spawn(NodeBundle {
            style: Style {
                flex_grow: 1.,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        })
        .set_parent(root)
        .id();

    for workspace in &e.0 {
        commands
            .spawn(TextBundle::from_section(workspace.name.clone(), default()))
            .set_parent(workspaces_root);

        setup_recursive(&mut commands, &colors, &workspace.child, 1., areas_root);
    }
}

fn setup_recursive(
    commands: &mut Commands,
    colors: &BColors,
    child: &Child,
    g: f32,
    parent: Entity,
) {
    match child {
        Child::Area { mode } => {
            let area_root = commands
                .spawn(NodeBundle {
                    background_color: colors.background.into(),
                    border_radius: BorderRadius::all(Val::Px(BORDER_RADIUS)),
                    style: Style {
                        flex_grow: g,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                })
                .insert(Pickable::default())
                .observe(|trigger: Trigger<Pointer<Down>>| {
                    print!("Down {}", trigger.entity());
                })
                .set_parent(parent)
                .id();
            let bar_root = commands
                .spawn(NodeBundle {
                    background_color: mode.color().with_luminance(0.3).into(),
                    border_radius: BorderRadius::top(Val::Px(BORDER_RADIUS)),
                    style: Style { ..default() },
                    ..default()
                })
                .set_parent(area_root)
                .insert(Pickable::default())
                .observe(|trigger: Trigger<Pointer<DragStart>>| {
                    print!("DragStart {}", trigger.entity());
                })
                .id();

            commands
                .spawn(TextBundle {
                    style: Style {
                        margin: UiRect::axes(Val::Px(3.), Val::Px(1.)),
                        ..default()
                    },
                    ..TextBundle::from_section(
                        mode.name(),
                        TextStyle {
                            font_size: 16.,
                            ..default()
                        },
                    )
                })
                .set_parent(bar_root);
        }
        Child::Split {
            direction,
            children,
        } => {
            let split_root = commands
                .spawn(NodeBundle {
                    style: Style {
                        flex_grow: g,
                        flex_direction: match direction {
                            SplitDirection::Horizontal => FlexDirection::Row,
                            SplitDirection::Vertical => FlexDirection::Column,
                        },
                        ..default()
                    },
                    ..default()
                })
                .set_parent(parent)
                .id();
            let count = children.len();
            for (i, (g, child)) in children.iter().enumerate() {
                setup_recursive(commands, colors, child, *g, split_root);

                // TODO Allow dragging on spacers to change the size of rows/columns
                // Add spacers
                if i < count - 1 {
                    commands
                        .spawn(NodeBundle {
                            // background_color: RED_100.into(),
                            style: match direction {
                                SplitDirection::Horizontal => Style {
                                    width: Val::Px(SPACING),
                                    height: Val::Percent(100.),
                                    // flex_grow: *g,
                                    // flex_basis: Val::Percent(100. * g),
                                    ..default()
                                },
                                SplitDirection::Vertical => Style {
                                    width: Val::Percent(100.),
                                    height: Val::Px(SPACING),
                                    // flex_grow: *g,
                                    // flex_basis: Val::Percent(100. * g),
                                    ..default()
                                },
                            },
                            ..default()
                        })
                        .set_parent(split_root);
                }
            }
        }
    }
}
