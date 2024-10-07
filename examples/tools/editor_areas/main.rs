#![allow(missing_docs)]

mod ui;

use bevy::{
    color::palettes::tailwind::*, prelude::*, render::view::cursor::CursorIcon,
    window::SystemCursorIcon,
};

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

#[derive(Resource)]
struct AreaRegistry {
    areas: Vec<Area>,
}

struct Area {
    name: String,
    color: Srgba,
}

// TODO Replace Mode eum with a registry for areas

#[derive(Clone, Copy)]
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

struct Workspace {
    name: String,
    child: Child,
}

struct Child {
    flex: f32,
    kind: ChildKind,
}

impl Child {
    fn new(flex: f32, kind: ChildKind) -> Self {
        Child { flex, kind }
    }

    // // fn find(&self, entity: Entity)
    // fn find_mut(&self, entity: Entity) -> &mut Child {

    // }
}

enum ChildKind {
    Area {
        mode: Mode,
        entity: Option<Entity>,
    },
    Split {
        direction: SplitDirection,
        children: Vec<Child>,
    },
}

impl ChildKind {
    fn area(mode: Mode) -> Self {
        ChildKind::Area { mode, entity: None }
    }
}

#[derive(Clone, Copy)]
enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy)]
enum Insertion {
    Above,
    Below,
    Left,
    Right,
}

impl Insertion {
    fn direction(&self) -> SplitDirection {
        match self {
            Insertion::Above | Insertion::Below => SplitDirection::Vertical,
            Insertion::Left | Insertion::Right => SplitDirection::Horizontal,
        }
    }
}

/// Must have at least 2 children. If reduced to one child it must be replaced with that child.
/// The flex_grow style property of its children should add up to at least one.
#[derive(Component, Clone)]
struct Split {
    direction: SplitDirection,
    flex_grow: f32,
}

#[derive(Component, Clone)]
struct Spacer {
    direction: SplitDirection,
}

#[derive(Component)]
struct AreaComponent {
    mode: Mode,
    flex_grow: f32,
}

#[derive(Resource, Default)]
struct WorkspacesSettings {
    workspaces: Vec<Workspace>,
    areas_root: Option<Entity>,
}

fn main() {
    let mut app = App::new();

    app.add_event::<RedoLayout>()
        .insert_resource(WorkspacesSettings {
            workspaces: vec![Workspace {
                name: "Default".to_string(),

                child: Child::new(
                    1.,
                    ChildKind::Split {
                        direction: SplitDirection::Horizontal,
                        children: vec![
                            Child::new(
                                0.15,
                                ChildKind::Split {
                                    direction: SplitDirection::Vertical,
                                    children: vec![
                                        Child::new(0.8, ChildKind::area(Mode::AreaD)),
                                        Child::new(0.2, ChildKind::area(Mode::AreaE)),
                                    ],
                                },
                            ),
                            Child::new(
                                0.7,
                                ChildKind::Split {
                                    direction: SplitDirection::Vertical,
                                    children: vec![
                                        Child::new(0.85, ChildKind::area(Mode::AreaA)),
                                        Child::new(0.15, ChildKind::area(Mode::AreaC)),
                                    ],
                                },
                            ),
                            Child::new(0.15, ChildKind::area(Mode::AreaB)),
                        ],
                    },
                ),
            }],
            ..default()
        });

    app.init_resource::<BColors>()
        .init_resource::<Dragging>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Editor".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                |mut query: Query<(&Split, &mut Style), Changed<Split>>| {
                    for (split, mut style) in &mut query {
                        style.flex_grow = split.flex_grow;
                    }
                },
                |mut query: Query<(&AreaComponent, &mut Style), Changed<AreaComponent>>| {
                    for (split, mut style) in &mut query {
                        style.flex_grow = split.flex_grow;
                    }
                },
                |mut workspaces_settings: ResMut<WorkspacesSettings>,
                 query: Query<Entity, With<AreaComponent>>,
                 colors: Res<BColors>,
                 mut commands: Commands,
                 mut reader: EventReader<RedoLayout>| {
                    if !reader.is_empty() {
                        reader.clear();
                    }

                    // Unparent all areas so they don't get despawned below
                    for entity in &query {
                        commands.entity(entity).remove_parent();
                    }

                    // Remove all entities in the layout
                    let areas_root = workspaces_settings.areas_root.unwrap();
                    commands.entity(areas_root).despawn_descendants();

                    // Redo layout
                    setup_recursive(
                        &mut commands,
                        &colors,
                        &mut workspaces_settings.workspaces[0].child,
                        areas_root,
                    );
                },
            ),
        );
    app.run();
}

#[derive(Event, Default)]
struct RedoLayout;

fn setup(
    mut workspace_settings: ResMut<WorkspacesSettings>,
    colors: Res<BColors>,
    mut commands: Commands,
) {
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
                ..default()
            },
            ..default()
        })
        .set_parent(root)
        .id();
    workspace_settings.areas_root = Some(areas_root);

    for workspace in &mut workspace_settings.workspaces {
        commands
            .spawn(TextBundle::from_section(workspace.name.clone(), default()))
            .set_parent(workspaces_root);

        setup_recursive(&mut commands, &colors, &mut workspace.child, areas_root);
    }
}

#[derive(Resource, Default)]
struct Dragging(Option<Entity>);

/// Marks this entity as bing part of the shadow dom.
#[derive(Component)]
struct ShadowDom;

/// Marks this entity as bing part of the real dom. Contains the id of the shadow dom entity
#[derive(Component)]
struct RealDom(Option<Entity>);

fn setup_shadow_dom_recursive(commands: &mut Commands, child: &mut Child, parent: Entity) {
    match &mut child.kind {
        ChildKind::Area { mode, entity } => {
            commands
                .spawn((
                    ShadowDom,
                    AreaComponent {
                        mode: *mode,
                        flex_grow: child.flex,
                    },
                ))
                .set_parent(parent);

            *entity = None;
        }
        ChildKind::Split {
            direction,
            children,
        } => {
            let split_root = create_split(commands, *direction, child.flex)
                .remove::<NodeBundle>()
                .insert(ShadowDom)
                .set_parent(parent)
                .id();
            for child in children.iter_mut() {
                setup_shadow_dom_recursive(commands, child, split_root);
            }
        }
    }
}

fn setup_recursive(commands: &mut Commands, colors: &BColors, child: &mut Child, parent: Entity) {
    match &mut child.kind {
        ChildKind::Area { mode, entity } => {
            if let Some(entity) = *entity {
                commands.entity(entity).set_parent(parent);
            } else {
                let area_root = commands
                    .spawn((
                        NodeBundle {
                            background_color: colors.background.into(),
                            border_radius: BorderRadius::all(Val::Px(BORDER_RADIUS)),
                            style: Style {
                                flex_grow: child.flex,
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            ..default()
                        },
                        AreaComponent {
                            mode: *mode,
                            flex_grow: child.flex,
                        },
                    ))
                    .observe(area_drag_drop)
                    .set_parent(parent)
                    .id();

                *entity = Some(area_root);

                let bar_root = commands
                    .spawn(NodeBundle {
                        background_color: mode.color().with_luminance(0.3).into(),
                        border_radius: BorderRadius::top(Val::Px(BORDER_RADIUS)),
                        style: Style { ..default() },
                        ..default()
                    })
                    .set_parent(area_root)
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
                    .observe(
                        move |_trigger: Trigger<Pointer<DragStart>>,
                              mut dragging: ResMut<Dragging>,
                              window_query: Query<Entity, With<Window>>,
                              mut commands: Commands| {
                            let window = window_query.single();
                            commands
                                .entity(window)
                                .insert(CursorIcon::System(SystemCursorIcon::Grabbing));
                            dragging.0 = Some(area_root);
                        },
                    )
                    .observe(
                        move |_trigger: Trigger<Pointer<DragEnd>>,
                              window_query: Query<Entity, With<Window>>,
                              mut commands: Commands| {
                            let window = window_query.single();
                            commands.entity(window).remove::<CursorIcon>();
                        },
                    )
                    .observe(move |_trigger: Trigger<Pointer<Click>>| {
                        println!("Click");
                    })
                    .observe(
                        |_trigger: Trigger<Pointer<Over>>,
                         window_query: Query<Entity, With<Window>>,
                         mut commands: Commands,
                         dragging: Res<Dragging>| {
                            if dragging.0.is_some() {
                                return;
                            }
                            let window = window_query.single();
                            commands
                                .entity(window)
                                .insert(CursorIcon::System(SystemCursorIcon::Pointer));
                        },
                    )
                    .observe(
                        |_trigger: Trigger<Pointer<Out>>,
                         window_query: Query<Entity, With<Window>>,
                         mut commands: Commands,
                         dragging: Res<Dragging>| {
                            if dragging.0.is_some() {
                                return;
                            }
                            let window = window_query.single();
                            commands.entity(window).remove::<CursorIcon>();
                        },
                    )
                    .set_parent(bar_root);
            }
        }
        ChildKind::Split {
            direction,
            children,
        } => {
            let split_root = create_split(commands, *direction, child.flex)
                .set_parent(parent)
                .id();
            let count = children.len();
            for (i, child) in children.iter_mut().enumerate() {
                setup_recursive(commands, colors, child, split_root);

                // TODO Allow dragging on spacers to change the size of rows/columns
                // Add spacers
                if i < count - 1 {
                    create_spacer(commands, *direction).set_parent(split_root);
                }
            }
        }
    }
}

fn area_drag_drop(
    trigger: Trigger<Pointer<DragDrop>>,
    mut dragging: ResMut<Dragging>,
    parent_query: Query<&Parent>,
    children_query: Query<&Children>,
    mut split_query: Query<&mut Split>,
    mut area_query: Query<&mut AreaComponent>,
    mut commands: Commands,
) {
    let target = trigger.entity();
    let Some(dropped) = dragging.0.take() else {
        return;
    };

    if target == dropped {
        return;
    }

    let old_parent = parent_query.get(dropped).unwrap().get();
    let old_siblings = children_query.get(old_parent).unwrap();
    // let cleanup_requires_flatten = old_siblings.len() == 3;
    let old_area_index = old_siblings.iter().position(|e| *e == dropped).unwrap();

    let mut target_area = area_query.get_mut(target).unwrap();
    target_area.flex_grow /= 2.;
    let new_flex_grow = target_area.flex_grow;

    let mut dropped_area = area_query.get_mut(dropped).unwrap();
    let old_flex_grow = dropped_area.flex_grow;
    dropped_area.flex_grow = new_flex_grow;

    let spacer_to_remove = if old_area_index == 0 {
        1
    } else {
        old_area_index - 1
    };

    // Move

    let parent = parent_query.get(target).unwrap().get();

    let same_parent = parent == old_parent;

    let siblings = children_query.get(parent).unwrap();
    let parent_split = split_query.get(parent).unwrap().clone();
    // let len = siblings.len();

    let target_index = siblings.iter().position(|e| *e == target).unwrap();

    // if index == 0 {
    let id = create_spacer(&mut commands, parent_split.direction).id();
    commands
        .entity(parent)
        .insert_children(target_index + 1, &[id, dropped]);
    // } else {
    // }

    // commands.entity(area).set_parent(parent);
    // commands.entity(old_parent);

    // Cleanup
    if cleanup_requires_flatten {
        let other_area = old_siblings[if old_area_index == 0 { 2 } else { 0 }];
        let grandparent = parent_query.get(old_parent).unwrap().get();
        let parent_siblings = children_query.get(grandparent).unwrap();
        let parent_index = parent_siblings
            .iter()
            .position(|e| *e == old_parent)
            .unwrap();
        if let Ok(mut area) = area_query.get_mut(other_area) {
            area.flex_grow = parent_split.flex_grow;
        } else if let Ok(mut split) = split_query.get_mut(other_area) {
            split.flex_grow = parent_split.flex_grow;
        }
        commands
            .entity(grandparent)
            .insert_children(parent_index, &[other_area]);
        commands.entity(old_parent).despawn_recursive();
    } else {
        commands
            .entity(old_siblings[spacer_to_remove])
            .despawn_recursive();
        let sibling_index = if old_area_index == 0 {
            2
        } else {
            old_area_index - 2
        };

        if let Ok(mut area) = area_query.get_mut(old_siblings[sibling_index]) {
            area.flex_grow += old_flex_grow;
        } else if let Ok(mut split) = split_query.get_mut(old_siblings[sibling_index]) {
            split.flex_grow += old_flex_grow;
        }
    }
}

fn create_split<'a>(
    commands: &'a mut Commands,
    direction: SplitDirection,
    flex_grow: f32,
) -> EntityCommands<'a> {
    commands.spawn((
        NodeBundle {
            style: Style {
                flex_grow,
                flex_direction: match direction {
                    SplitDirection::Horizontal => FlexDirection::Row,
                    SplitDirection::Vertical => FlexDirection::Column,
                },
                ..default()
            },
            ..default()
        },
        Split {
            direction,
            flex_grow,
        },
    ))
}

fn create_spacer<'a>(commands: &'a mut Commands, direction: SplitDirection) -> EntityCommands<'a> {
    let mut ec = commands.spawn((
        NodeBundle {
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
        },
        Spacer { direction },
    ));
    ec.observe(
        |trigger: Trigger<Pointer<Over>>,
         query: Query<&Spacer>,
         window_query: Query<Entity, With<Window>>,
         mut commands: Commands,
         dragging: Res<Dragging>| {
            if dragging.0.is_some() {
                return;
            }
            let window = window_query.single();
            let this = query.get(trigger.entity()).unwrap();
            commands
                .entity(window)
                .insert(CursorIcon::System(match this.direction {
                    SplitDirection::Horizontal => SystemCursorIcon::EwResize,
                    SplitDirection::Vertical => SystemCursorIcon::NsResize,
                }));
        },
    )
    .observe(
        |_trigger: Trigger<Pointer<Out>>,
         window_query: Query<Entity, With<Window>>,
         mut commands: Commands,
         dragging: Res<Dragging>| {
            if dragging.0.is_some() {
                return;
            }
            let window = window_query.single();
            commands
                .entity(window)
                .insert(CursorIcon::System(SystemCursorIcon::Default));
        },
    );
    ec
}
