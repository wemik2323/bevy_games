#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use bevy::{prelude::*, window::PrimaryWindow};

const TARGET_RADUIS: f32 = 50.0;
const TARGET_DIAMETER: f32 = 100.0;

fn main() {
    App::new()
        .add_systems(Startup, setup)
        .add_systems(Update, update_targets)
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(1., 1., 1.)))
        .init_resource::<Counter>()
        .run();
}

#[derive(Resource, Default)]
struct Counter(u32, u32);

#[derive(Component)]
struct Target;

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.spawn(Camera2d);
    cmd.spawn(target(&assets, 0., 0.));
    cmd.spawn((
        Text::new("Kill count: 0\nClicks count: 0"),
        TextColor(Color::BLACK),
    ));
}

fn update_targets(
    mut cmd: Commands,
    target_query: Query<(Entity, &Transform), With<Target>>,
    main_window_query: Query<&Window, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    assets: Res<AssetServer>,
    mut counter: ResMut<Counter>,
    mut text_query: Query<&mut Text>,
) {
    let Ok((entity, target_transform)) = target_query.single() else {
        return;
    };
    let main_window = main_window_query.single().unwrap();

    let Some(mut cursor_position) = main_window.cursor_position() else {
        return;
    };

    let Ok(mut counter_text) = text_query.single_mut() else {
        return;
    };

    cursor_position.y *= -1.;
    cursor_position.y += main_window.size().y / 2.;
    cursor_position.x -= main_window.size().x / 2.;

    let distance = cursor_position.distance(target_transform.translation.truncate());

    if mouse.just_pressed(MouseButton::Left) {
        if distance <= TARGET_RADUIS {
            counter.0 += 1;

            cmd.entity(entity).despawn();
            let x = rand::random_range(
                -main_window.size().x / 2. + TARGET_DIAMETER
                    ..=main_window.size().x / 2. - TARGET_DIAMETER,
            );
            let y = rand::random_range(
                -main_window.size().y / 2. + TARGET_DIAMETER
                    ..=main_window.size().y / 2. - TARGET_DIAMETER,
            );

            cmd.spawn(target(&assets, x, y));
        }
        counter.1 += 1;

        counter_text.0 = format!("Kill count: {}\nClicks count: {}", counter.0, counter.1);
    }

}

fn target(assets: &AssetServer, x_pos: f32, y_pos: f32) -> impl Bundle {
    (
        Sprite {
            image: assets.load("target.png"),
            custom_size: Some(Vec2::new(TARGET_DIAMETER, TARGET_DIAMETER)),
            ..default()
        },
        Target,
        Transform::from_xyz(x_pos, y_pos, 0.),
    )
}
