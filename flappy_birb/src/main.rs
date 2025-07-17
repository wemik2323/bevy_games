#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResolution},
};

const GRAVITY: f32 = -9.8;
const JUMP_FORCE: f32 = 5.0;
const PIPE_SPEED: f32 = 150.0;
const PIPE_WIDTH: f32 = 100.0;
const BIRB_WIDTH: f32 = 63.0;
const BIRB_HEIGHT: f32 = 44.5;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Window {
                    resolution: WindowResolution::new(600., 600.),
                    position: WindowPosition::Centered(MonitorSelection::Primary),
                    title: "Flappy Birb".into(),
                    resizable: false,
                    ..default()
                }
                .into(),
                ..default()
            }),
        )
        .add_event::<GameOverEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_birb,
                spawn_pipes,
                update_pipes,
                handle_game_over,
                update_score,
            ),
        )
        .insert_resource(ClearColor(Color::srgb(1., 1., 1.)))
        .insert_resource(GameArgs {
            is_playing: false,
            score: 0,
            pipe_spawn_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        })
        .run();
}

#[derive(Component)]
struct Birb {
    velocity: f32,
}

#[derive(Component)]
struct Pipe {
    passed: bool,
    size: Vec2,
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct GameStartText;

#[derive(Resource)]
struct GameArgs {
    is_playing: bool,
    score: u32,
    pipe_spawn_timer: Timer,
}

#[derive(Event)]
struct GameOverEvent;

fn setup(assets: Res<AssetServer>, mut cmd: Commands) {
    cmd.spawn(Camera2d);
    cmd.spawn((
        Sprite {
            image: assets.load("birb.png"),
            custom_size: Some(Vec2::new(BIRB_WIDTH, BIRB_HEIGHT)),
            ..default()
        },
        Birb { velocity: 0.0 },
    ));

    cmd.spawn((Text::new("Score: 0"), TextColor(Color::BLACK), ScoreText));

    cmd.spawn((
        Text::new("Press SPACE to start"),
        TextColor(Color::BLACK),
        Node {
            left: Val::Px(200.),
            top: Val::Px(330.),
            ..default()
        },
        GameStartText,
    ));
}

fn update_birb(
    main_window_query: Query<&Window, With<PrimaryWindow>>,
    mut birb_query: Query<(&mut Transform, &mut Birb)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut game_start_text_query: Query<&mut Text, With<GameStartText>>,
    mut game_args: ResMut<GameArgs>,
    mut game_over_events: EventWriter<GameOverEvent>,
) {
    let Ok(main_window) = main_window_query.single() else {
        return;
    };

    let Ok(mut game_start_text) = game_start_text_query.single_mut() else {
        return;
    };

    if let Ok((mut transform_birb, mut birb)) = birb_query.single_mut() {
        if keyboard_input.just_pressed(KeyCode::Space) {
            if !game_args.is_playing {
                game_args.is_playing = true;
                game_start_text.0 = format! {""};
            }
            birb.velocity = JUMP_FORCE;
        }

        if game_args.is_playing {
            birb.velocity += GRAVITY * time.delta_secs();

            transform_birb.translation.y += birb.velocity * 100.0 * time.delta_secs();

            if transform_birb.translation.y > main_window.size().y / 2.
                || transform_birb.translation.y < -main_window.size().y / 2.
            {
                game_over_events.write(GameOverEvent);
            }
        }
    }
}

fn spawn_pipes(mut commands: Commands, time: Res<Time>, mut game_args: ResMut<GameArgs>) {
    if !game_args.is_playing {
        return;
    }

    game_args.pipe_spawn_timer.tick(time.delta());
    if game_args.pipe_spawn_timer.just_finished() {
        let pipe_size_y = rand::random_range(100.0..=400.0);

        commands.spawn((
            Sprite {
                color: Color::srgb(0., 1., 0.),
                custom_size: Some(Vec2::new(PIPE_WIDTH, pipe_size_y)),
                ..default()
            },
            Transform::from_xyz(300.0, -300.0, 0.0),
            Pipe {
                passed: false,
                size: Vec2 {
                    x: PIPE_WIDTH,
                    y: pipe_size_y,
                },
            },
        ));

        commands.spawn((
            Sprite {
                color: Color::srgb(0., 1., 0.),
                custom_size: Some(Vec2::new(PIPE_WIDTH, 600.0 - pipe_size_y)),
                ..default()
            },
            Transform::from_xyz(300.0, 300.0, 0.0),
            Pipe {
                passed: false,
                size: Vec2 {
                    x: PIPE_WIDTH,
                    y: 600.0 - pipe_size_y,
                },
            },
        ));
    }
}

fn update_pipes(
    mut commands: Commands,
    mut pipe_query: Query<(Entity, &mut Transform, &mut Pipe)>,
    mut game_args: ResMut<GameArgs>,
    birb_query: Query<&Transform, (With<Birb>, Without<Pipe>)>,
    time: Res<Time>,
    mut game_over_events: EventWriter<GameOverEvent>,
) {
    if !game_args.is_playing {
        return;
    }

    let Ok(birb_transform) = birb_query.single() else {
        return;
    };

    for (pipe_entity, mut pipe_transform, mut pipe) in pipe_query.iter_mut() {
        pipe_transform.translation.x -= PIPE_SPEED * time.delta_secs();

        if birb_transform.translation.x - BIRB_WIDTH / 2.0
            < pipe_transform.translation.x + PIPE_WIDTH / 2.0
            && birb_transform.translation.x + BIRB_WIDTH / 2.0
                > pipe_transform.translation.x - PIPE_WIDTH / 2.0
            && birb_transform.translation.y - BIRB_HEIGHT / 2.0
                < pipe_transform.translation.y + pipe.size.y / 2.0
            && birb_transform.translation.y + BIRB_HEIGHT / 2.0
                > pipe_transform.translation.y - pipe.size.y / 2.0
        {
            game_over_events.write(GameOverEvent);
        }

        if !pipe.passed
            && pipe_transform.translation.x + PIPE_WIDTH / 2.0
                < birb_transform.translation.x - BIRB_WIDTH / 2.0
        {
            pipe.passed = true;
            game_args.score += 1;
        }

        if pipe_transform.translation.x < -300.0 - PIPE_WIDTH {
            commands.entity(pipe_entity).despawn();
        }
    }
}

fn handle_game_over(
    mut commands: Commands,
    mut game_args: ResMut<GameArgs>,
    mut game_start_text_query: Query<&mut Text, With<GameStartText>>,
    game_over_events: EventReader<GameOverEvent>,
    birb_query: Query<Entity, With<Birb>>,
    pipe_query: Query<Entity, With<Pipe>>,
    mut score_text_query: Query<&mut Text, (With<ScoreText>, Without<GameStartText>)>,
) {
    if game_over_events.is_empty() {
        return;
    }

    let Ok(mut game_over_text) = game_start_text_query.single_mut() else {
        return;
    };

    let final_score = game_args.score/2;

    game_over_text.0 = format!(
        "Game Over! Score: {}\nPress SPACE to restart",
        final_score
    );

    game_args.is_playing = false;

    for pipe_entity in pipe_query.iter() {
        commands.entity(pipe_entity).despawn();
    }

    if let Ok(bird_entity) = birb_query.single() {
        commands.entity(bird_entity).insert((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Birb {
                velocity: JUMP_FORCE,
            },
        ));
    }

    game_args.score = 0;

    if let Ok(mut text) = score_text_query.single_mut() {
        text.0 = format!("Score: 0");
    }
}

fn update_score(
    game_args: Res<GameArgs>,
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
) {
    if let Ok(mut text) = score_text_query.single_mut() {
        text.0 = format!("Score: {}", game_args.score/2);
    }
}
