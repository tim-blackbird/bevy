use std::f32::consts::TAU;

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    input::mouse::MouseMotion,
    math::Vec2Swizzles,
    prelude::*,
    window::PresentMode,
};

const SYSTEM_COUNT: u32 = 1;

fn main() {
    let mut app = App::new();
    app//.insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Many Debug Lines".to_string(),
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin,
        ))
        .insert_resource(Config {
            line_count: 1,
            fancy: false,
        })
        .insert_resource(GizmoConfig {
            line_width: 5.,
            // line_perspective: true,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (input, ui_system, camera_control));

    for _ in 0..SYSTEM_COUNT {
        app.add_systems(Update, system);
    }

    app.run();
}

#[derive(Resource, Debug)]
struct Config {
    line_count: u32,
    fancy: bool,
}

fn input(mut config: ResMut<Config>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Up) {
        config.line_count += 10_000;
    }
    if input.just_pressed(KeyCode::Down) {
        config.line_count = config.line_count.saturating_sub(10_000);
    }
    if input.just_pressed(KeyCode::Space) {
        config.fancy = !config.fancy;
    }
}

fn system(config: Res<Config>, time: Res<Time>, mut draw: Gizmos) {
    if !config.fancy {
        for _ in 0..(config.line_count / SYSTEM_COUNT) {
            // draw.line(Vec3::NEG_ONE * 15., Vec3::ONE * 15., Color::BLACK);
            draw.line(Vec3::NEG_X * 150., Vec3::X * 150., Color::YELLOW);
        }
    } else {
        for i in 0..(config.line_count / SYSTEM_COUNT) {
            let angle = i as f32 / (config.line_count / SYSTEM_COUNT) as f32 * TAU;

            let vector = Vec2::from(angle.sin_cos()).extend(time.elapsed_seconds().sin());
            let start_color = Color::rgb(vector.x, vector.z, 0.5);
            let end_color = Color::rgb(-vector.z, -vector.y, 0.5);

            draw.line_gradient(vector, -vector, start_color, end_color);
        }
    }
}

fn camera_control(
    mut camera: Query<&mut Transform, With<Camera>>,
    mut mouse_deltas: EventReader<MouseMotion>,
    mut euler: Local<Vec3>,
) {
    let mut camera_transform = camera.single_mut();

    *euler -= mouse_deltas
        .iter()
        .map(|e| e.delta)
        .sum::<Vec2>()
        .yx()
        .extend(0.)
        * 0.005;

    euler.x = euler.x.clamp(-TAU / 4., TAU / 4.);

    camera_transform.rotation = Quat::from_rotation_y(euler.y) * Quat::from_rotation_x(euler.x);
    camera_transform.translation = camera_transform.back() * 13.;
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    warn!(include_str!("warning_string.txt"));

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(3., 1., 5.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(TextBundle::from_section(
        "",
        TextStyle {
            font_size: 30.,
            ..default()
        },
    ));
}

fn ui_system(
    mut query: Query<&mut Text>,
    mut peen: Query<&Transform, With<Camera>>,
    config: Res<Config>,
    diag: Res<DiagnosticsStore>,
) {
    let mut text = query.single_mut();

    let Some(fps) = diag.get(FrameTimeDiagnosticsPlugin::FPS).and_then(|fps| fps.smoothed()) else {
        return;
    };

    let c = peen.single();
    text.sections[0].value = format!(
        "Line count: {}\n\
        FPS: {:.0}\n\n\
        Controls:\n\
        Up/Down: Raise or lower the line count.\n\
        Spacebar: Toggle fancy mode.\n\n\
        {:.1}",
        config.line_count,
        fps,
        Vec3::Y.angle_between(c.forward()).to_degrees()
    );
}
