#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResolution},
};

const MINEFILED_WIDTH: usize = 10;
const MINEFILED_HEIGHT: usize = 10;
const BOMBS_AMOUNT: usize = 13;
const CELL_SIZE: f32 = 50.;

#[derive(States, Eq, PartialEq, Hash, Debug, Clone, Default)]
enum GameStates {
    #[default]
    Playing,
    Loss,
    Win,
}

#[derive(Component, Clone)]
struct Cell {
    x: f32,
    y: f32,
    is_mined: bool,
    is_open: bool,
    is_tagged: bool,
    mines_around: u32,
}

impl Cell {
    fn new(
        x: f32,
        y: f32,
        is_mined: bool,
        is_open: bool,
        is_tagged: bool,
        mines_around: u32,
    ) -> Cell {
        Self {
            x,
            y,
            is_mined,
            is_open,
            is_tagged,
            mines_around,
        }
    }
}

#[derive(Component)]
struct ControlPanel;

#[derive(Component)]
struct MineField {
    cells: Vec<Vec<Cell>>,
}

#[derive(Resource)]
struct CellTextures {
    hidden: Handle<Image>,
    revealed: Handle<Image>,
    mine: Handle<Image>,
    flag: Handle<Image>,
    numbers: [Handle<Image>; 8],
}

#[derive(Resource)]
struct ControlPanelTextures {
    casual: Handle<Image>,
    scared: Handle<Image>,
    dead: Handle<Image>,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Window {
                    resolution: WindowResolution::new(600., 800.),
                    position: WindowPosition::Centered(MonitorSelection::Primary),
                    title: "Minesweeper".into(),
                    resizable: false,
                    ..default()
                }
                .into(),
                ..default()
            }),
        )
        .insert_resource(ClearColor(Color::srgb(1., 1., 1.)))
        .init_state::<GameStates>()
        .add_systems(Startup, (setup, setup_textures))
        .add_systems(Update, (draw_minefield, draw_control_panel, input_listener))
        .run();
}

fn setup(mut cmd: Commands) {
    cmd.spawn(Camera2d);
    cmd.spawn((
        Sprite {
            color: Color::srgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::new(550., 550.)),
            ..default()
        },
        Transform::from_xyz(0., -80., 0.),
        MineField {
            cells: generate_minefield(),
        },
    ));
    cmd.spawn((
        Sprite {
            color: Color::srgb(0., 0., 0.),
            custom_size: Some(Vec2::new(550., 150.)),
            ..default()
        },
        Transform::from_xyz(0., 300., 0.),
        ControlPanel,
    ));
}

fn setup_textures(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.insert_resource(CellTextures {
        hidden: assets.load("cells/unknown.png"),
        revealed: assets.load("cells/empty.png"),
        mine: assets.load("cells/bomb.png"),
        flag: assets.load("cells/flag.png"),
        numbers: [
            assets.load("cells/1.png"),
            assets.load("cells/2.png"),
            assets.load("cells/3.png"),
            assets.load("cells/4.png"),
            assets.load("cells/5.png"),
            assets.load("cells/6.png"),
            assets.load("cells/7.png"),
            assets.load("cells/8.png"),
        ],
    });
    cmd.insert_resource(ControlPanelTextures {
        casual: assets.load("control_panel/casual_face.png"),
        scared: assets.load("control_panel/scared_face.png"),
        dead: assets.load("control_panel/dead_face.png"),
    });
}

fn generate_minefield() -> Vec<Vec<Cell>> {
    let mut cells = vec![
        vec![Cell::new(0.0, 0.0, false, false, false, 0); MINEFILED_WIDTH];
        MINEFILED_HEIGHT
    ];

    let mut i: u32 = 0;
    while i <= BOMBS_AMOUNT as u32 {
        let row = rand::random_range(0..MINEFILED_WIDTH);
        let column = rand::random_range(0..MINEFILED_HEIGHT);

        if !cells[row][column].is_mined {
            cells[row][column].is_mined = true;
            i += 1;
        }
    }

    for row in 0..MINEFILED_WIDTH {
        for column in 0..MINEFILED_HEIGHT {
            if !cells[row][column].is_mined {
                continue;
            }
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    let new_row = row as i32 + dx;
                    let new_column = column as i32 + dy;

                    if new_row >= 0
                        && new_row < MINEFILED_WIDTH as i32
                        && new_column >= 0
                        && new_column < MINEFILED_HEIGHT as i32
                    {
                        cells[new_row as usize][new_column as usize].mines_around += 1;
                    }
                }
            }
        }
    }

    let start_x = -((MINEFILED_WIDTH as f32 * CELL_SIZE) / 2.0) + CELL_SIZE / 2.0;
    let start_y = -((MINEFILED_HEIGHT as f32 * CELL_SIZE) / 2.0) + CELL_SIZE / 2.0 - 80.0;

    for row in 0..MINEFILED_HEIGHT {
        for column in 0..MINEFILED_WIDTH {
            cells[row][column].x = start_x + column as f32 * CELL_SIZE;
            cells[row][column].y = start_y + row as f32 * CELL_SIZE;
        }
    }

    cells
}

fn draw_minefield(mut cmd: Commands, query: Query<&MineField>, textures: Res<CellTextures>, cell_sprites: Query<Entity, With<Cell>>) {
    let Ok(minefield) = query.single() else {
        return;
    };

    for entity in cell_sprites.iter() {
        cmd.entity(entity).despawn();
    }

    for row in &minefield.cells {
        for cell in row {
            let texture = match (cell.is_open, cell.is_tagged, cell.is_mined) {
                (false, true, _) => &textures.flag,
                (true, _, true) => &textures.mine,
                (true, _, false) if cell.mines_around > 0 => {
                    &textures.numbers[cell.mines_around as usize - 1]
                }
                (true, _, false) => &textures.revealed,
                _ => &textures.hidden,
            };

            cmd.spawn((
                Sprite {
                    image: texture.clone(),
                    color: Color::srgb(0.7, 0.7, 0.7),
                    custom_size: Some(Vec2::splat(CELL_SIZE * 0.9)),
                    ..default()
                },
                Transform::from_xyz(cell.x, cell.y, 1.),
            ));
        }
    }
}

fn draw_control_panel(
    mut cmd: Commands,
    game_state: Res<State<GameStates>>,
    textures: Res<ControlPanelTextures>,
    panel_sprites: Query<Entity, With<ControlPanel>>,
) {
    let texture = match game_state.get() {
        GameStates::Playing => &textures.casual,
        GameStates::Loss => &textures.dead,
        GameStates::Win => &textures.scared,
    };

    for entity in panel_sprites {
        cmd.entity(entity).despawn();
    }

    cmd.spawn((
        Sprite {
            image: texture.clone(),
            color: Color::srgb(0.7, 0.7, 0.7),
            custom_size: Some(Vec2::new(150., 150.)),
            ..default()
        },
        Transform::from_xyz(0., 300., 2.),
    ));
}

fn input_listener(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mouse_btn: Res<ButtonInput<MouseButton>>,
    game_state: Res<State<GameStates>>,
    mut next_state: ResMut<NextState<GameStates>>,
    mut minefield_query: Query<&mut MineField>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let window_size = Vec2::new(window.width(), window.height());
    let cursor_world_pos = Vec2::new(
        cursor_pos.x - window_size.x / 2.0,
        window_size.y / 2.0 - cursor_pos.y,
    );

    let Ok(mut minefield) = minefield_query.single_mut() else {
        return;
    };

    // Control pannel
    if mouse_btn.just_pressed(MouseButton::Left)
        && cursor_world_pos.distance(Vec2::new(0.0, 300.0)) < 75.0
    {
        *minefield = MineField {
            cells: generate_minefield(),
        };
        next_state.set(GameStates::Playing);
        return;
    }

    // Open cells
    if *game_state.get() == GameStates::Playing {
        if mouse_btn.just_pressed(MouseButton::Left) {
            if let Some((row, col)) = find_cell_under_cursor(&minefield, cursor_world_pos) {
                let cell = &minefield.cells[row][col];

                if !cell.is_open && !cell.is_tagged {
                    if cell.is_mined {
                        reveal_all_mines(&mut minefield);
                        next_state.set(GameStates::Loss);
                        return;
                    }

                    if cell.mines_around == 0 {
                        reveal_cells(&mut minefield, col, row);
                    } else {
                        minefield.cells[row][col].is_open = true;
                    }
                } else if cell.is_open && cell.mines_around > 0 {
                    try_reveal_around_number(&mut minefield, row, col, &mut next_state);
                }
            }
        }

        // Place flags
        if mouse_btn.just_pressed(MouseButton::Right) {
            if let Some((row, col)) = find_cell_under_cursor(&minefield, cursor_world_pos) {
                if !minefield.cells[row][col].is_open {
                    minefield.cells[row][col].is_tagged = !minefield.cells[row][col].is_tagged;
                }
            }
        }

        check_win_condition(&minefield, &mut next_state);
    }
}

fn find_cell_under_cursor(minefield: &MineField, cursor_pos: Vec2) -> Option<(usize, usize)> {
    for row in 0..MINEFILED_HEIGHT {
        for col in 0..MINEFILED_WIDTH {
            let cell = &minefield.cells[row][col];
            if (cursor_pos.x - cell.x).abs() < CELL_SIZE / 2.0
                && (cursor_pos.y - cell.y).abs() < CELL_SIZE / 2.0
            {
                return Some((row, col));
            }
        }
    }
    None
}

fn reveal_all_mines(minefield: &mut MineField) {
    for row in &mut minefield.cells {
        for cell in row {
            if cell.is_mined {
                cell.is_open = true;
            }
        }
    }
}

fn try_reveal_around_number(
    minefield: &mut MineField,
    row: usize,
    col: usize,
    next_state: &mut ResMut<NextState<GameStates>>,
) {
    let cell = &minefield.cells[row][col];
    if !cell.is_open || cell.mines_around == 0 {
        return;
    }

    let mut flagged = 0;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = col as i32 + dx;
            let ny = row as i32 + dy;

            if nx >= 0
                && nx < MINEFILED_WIDTH as i32
                && ny >= 0
                && ny < MINEFILED_HEIGHT as i32
            {
                if minefield.cells[ny as usize][nx as usize].is_tagged {
                    flagged += 1;
                }
            }
        }
    }

    if flagged == cell.mines_around {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = col as i32 + dx;
                let ny = row as i32 + dy;

                if nx >= 0
                    && nx < MINEFILED_WIDTH as i32
                    && ny >= 0
                    && ny < MINEFILED_HEIGHT as i32
                {
                    let x = nx as usize;
                    let y = ny as usize;

                    if !minefield.cells[y][x].is_open && !minefield.cells[y][x].is_tagged {
                        if minefield.cells[y][x].mines_around == 0 {
                            reveal_cells(minefield, x, y);
                        }

                        minefield.cells[y][x].is_open = true;

                        if minefield.cells[y][x].is_mined {
                            reveal_all_mines(minefield);
                            next_state.set(GameStates::Loss);
                            return;
                        }
                    }
                }
            }
        }
    }
}

fn check_win_condition(minefield: &MineField, next_state: &mut ResMut<NextState<GameStates>>) {
    let mut all_cleared = true;
    for row in &minefield.cells {
        for cell in row {
            if !cell.is_mined && !cell.is_open {
                all_cleared = false;
                break;
            }
        }
        if !all_cleared {
            break;
        }
    }
    if all_cleared {
        next_state.set(GameStates::Win);
    }
}

fn reveal_cells(minefield: &mut MineField, x: usize, y: usize) {
    if y >= MINEFILED_HEIGHT
        || x >= MINEFILED_WIDTH
        || minefield.cells[y][x].is_open
        || minefield.cells[y][x].is_tagged
        || minefield.cells[y][x].is_mined
    {
        return;
    }

    minefield.cells[y][x].is_open = true;

    if minefield.cells[y][x].mines_around == 0 {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0
                    && nx < MINEFILED_WIDTH as i32
                    && ny >= 0
                    && ny < MINEFILED_HEIGHT as i32
                {
                    reveal_cells(minefield, nx as usize, ny as usize);
                }
            }
        }
    }
}
