mod audio;
mod holes;
mod player;
mod star;

use audio::*;
use bevy::{
    asset::AssetMetaCheck, audio::PlaybackSettings, color::palettes::css::DARK_GREY, prelude::*,
    sprite::AlphaMode2d,
};
use bevy_audio_controller::prelude::*;
use bevy_rapier2d::prelude::*;
use holes::*;
use player::*;
use star::*;

#[derive(Component, Default, AudioChannel, Reflect)]
struct SfxChannel;

#[derive(Component)]
struct Health;

#[derive(Event)]
enum ButtonClicked {
    ExitGame,
    StartGame,
    PauseGame,
    ResumeGame,
    RestartGame,
}

#[derive(Component)]
struct ScoreText;

#[derive(Resource, States, Debug, Hash, PartialEq, Eq, Clone)]
enum GameState {
    Start,
    Playing,
    GameOver,
    Paused,
}
impl Default for GameState {
    fn default() -> Self {
        GameState::Start
    }
}

#[derive(Component)]
struct StartButton;

#[derive(Component)]
struct MainMenu;

#[derive(Component)]
struct GameOverMenu;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..AssetPlugin::default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(AudioControllerPlugin)
        .add_systems(Startup, (setup).chain())
        .add_plugins(PlayerPlugin)
        .add_plugins(StarPlugin)
        .add_plugins(HolePlugin)
        .add_plugins(BackgroundAudioPlugin)
        .register_audio_channel::<SfxChannel>()
        .insert_state(GameState::Start)
        .init_resource::<GameState>()
        //.configure_sets(Update, RollyPollySet.run_if(in_state(GameState::Playing)))
        .add_systems(
            Update,
            (button_system, button_events)
                .chain()
                .run_if(in_state(GameState::Paused)),
        )
        .add_systems(OnEnter(GameState::Start), (setup_game, spawn_score_text))
        .add_systems(
            Update,
            (game_over_menu, button_events, button_system)
                .chain()
                .run_if(in_state(GameState::GameOver)),
        )
        .add_systems(
            Update,
            (update_score_text,)
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_event::<ButtonClicked>()
        .run();
}

fn setup(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(GameState::Start);
    commands.insert_resource(GameState::GameOver);
    commands.insert_resource(GameState::Paused);
    commands.insert_resource(GameState::Playing);
    commands.spawn(Camera2d::default());
    start_menu(&mut commands, asset_server);
    next_state.set(GameState::Paused);
}

fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window: Query<&Window>,
    mut next_state: ResMut<NextState<GameState>>,
    health: Query<Entity, With<Health>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let window = window.single();
    let texture_handle = asset_server.load("space.png");
    let mesh_handle = meshes.add(Rectangle::from_size(Vec2::new(
        window.width(),
        window.height(),
    )));

    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;
    let thickness = 10.0; // thickness for the boundaries

    // Top boundary
    commands.spawn((
        Transform::from_xyz(0.0, (half_height - 40.) + thickness / 2.0, 0.0),
        GlobalTransform::default(),
        RigidBody::Fixed,
        Collider::cuboid(half_width, thickness / 2.0),
    ));

    // Bottom boundary
    commands.spawn((
        Transform::from_xyz(0.0, -half_height - thickness / 2.0, 0.0),
        GlobalTransform::default(),
        RigidBody::Fixed,
        Collider::cuboid(half_width, thickness / 2.0),
    ));

    // Left boundary
    commands.spawn((
        Transform::from_xyz(-half_width - thickness / 2.0, 0.0, 0.0),
        GlobalTransform::default(),
        RigidBody::Fixed,
        Collider::cuboid(thickness / 2.0, half_height),
    ));

    // Right boundary
    commands.spawn((
        Transform::from_xyz(half_width + thickness / 2.0, 0.0, 0.0),
        GlobalTransform::default(),
        RigidBody::Fixed,
        Collider::cuboid(thickness / 2.0, half_height),
    ));

    // Background
    commands.spawn((
        Mesh2d(mesh_handle.clone()),
        MeshMaterial2d(materials.add(ColorMaterial {
            color: DARK_GREY.into(),
            alpha_mode: AlphaMode2d::Mask(0.9),
            texture: Some(texture_handle.clone()),
        })),
        Transform::from_translation(Vec3::new(0., -50., -1.0)),
    ));

    for health in health.iter() {
        commands.entity(health).despawn_recursive();
    }

    // Spawn health indicators
    for i in 0..3 {
        let position = Vec3::new(
            window.width() / 2.0 - 20.0 - (i as f32 * 35.0),
            window.height() / 2.0 - 20.0,
            0.,
        );

        commands.spawn((
            Sprite::from_image(asset_server.load("ball_blue_small.png")),
            Transform::from_translation(position),
            Health,
        ));
    }
    next_state.set(GameState::Playing);
}

fn spawn_score_text(mut commands: Commands) {
    commands.spawn((
        Text::new("Start: 0"),
        TextFont {
            font_size: 20.,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        },
        ScoreText,
    ));
}

fn start_menu(commands: &mut Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Relative,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            MainMenu,
        ))
        .with_children(|parent| {
            // Image
            parent
                .spawn((Node {
                    width: Val::Auto,
                    height: Val::Auto,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(5.0)), // Add some space below the image
                    ..default()
                },))
                .with_child((
                    ImageNode {
                        image: asset_server.load("title.png"),
                        ..ImageNode::default()
                    },
                    Transform::from_scale(Vec3::splat(0.7)), // Scale down the image
                ));

            // Start Game Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Auto,
                        height: Val::Auto,
                        padding: UiRect::all(Val::Px(10.)),
                        border: UiRect::all(Val::Px(5.0)),
                        margin: UiRect::bottom(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BorderRadius::MAX,
                    BackgroundColor(NORMAL_BUTTON),
                    StartButton,
                ))
                .with_child((
                    Text::new("Start Game"),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));

            // Quit Game Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Auto,
                        height: Val::Auto,
                        padding: UiRect::all(Val::Px(10.)),
                        border: UiRect::all(Val::Px(5.0)),
                        margin: UiRect::bottom(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BorderRadius::MAX,
                    BackgroundColor(NORMAL_BUTTON),
                ))
                .with_child((
                    Text::new("Quit Game"),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
        });
}

fn game_over_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Relative,
                display: Display::Grid,
                grid_template_rows: RepeatedGridTrack::auto(2), // 2 auto rows
                row_gap: Val::Px(4.),
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    top: Val::Auto,
                    bottom: Val::Auto,
                },
                padding: UiRect::all(Val::Px(10.0)),
                width: Val::Percent(30.0),
                height: Val::Percent(30.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::hsla(1., 1., 1., 0.8)),
            GameOverMenu,
        ))
        .with_children(|parent| {
            // First row - Quit Game button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Auto,
                        height: Val::Auto,
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BorderRadius::MAX,
                    BackgroundColor(NORMAL_BUTTON),
                ))
                .with_child((
                    Text::new("Quit Game"),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));

            // Second row - Start Over button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Auto,
                        height: Val::Auto,
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BorderRadius::MAX,
                    BackgroundColor(NORMAL_BUTTON),
                ))
                .with_child((
                    Text::new("Start Over"),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
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
    mut event_writter: EventWriter<ButtonClicked>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    text: Query<&Text>,
    mut sfx_play_ew: EventWriter<GlobalPlayEvent>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let text = match text.get(children[0]) {
            Ok(text) => text,
            Err(err) => {
                info!("Error: {:?}", err);
                return;
            }
        };
        let button_event = match text.as_str() {
            "Quit Game" => ButtonClicked::ExitGame,
            "Start Game" => ButtonClicked::StartGame,
            "Pause Game" => ButtonClicked::PauseGame,
            "Resume Game" => ButtonClicked::ResumeGame,
            "Start Over" => ButtonClicked::RestartGame,
            err => panic!("Unknown button text {}", err),
        };

        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::WHITE;
                event_writter.send(button_event);
                let event = GlobalPlayEvent::new(AudioFiles::ButtonClickOGG)
                    .with_settings(PlaybackSettings::ONCE);
                sfx_play_ew.send(event);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
                commands.spawn((
                    PlaybackSettings::ONCE,
                    AudioPlayer::<AudioSource>(asset_server.load("rollover.ogg")),
                ));
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn button_events(
    mut event: EventReader<ButtonClicked>,
    mut next_state: ResMut<NextState<GameState>>,
    main_menu: Query<Entity, With<MainMenu>>,
    game_over_menu: Query<Entity, With<GameOverMenu>>,
    mut commands: Commands,
) {
    for button in event.read() {
        match button {
            ButtonClicked::StartGame => {
                next_state.set(GameState::Playing);

                match main_menu.get_single() {
                    Ok(main_menu) => {
                        commands.entity(main_menu).despawn_recursive();
                    }
                    Err(err) => {
                        info!("Error: {:?}", err);
                    }
                };
            }
            ButtonClicked::ExitGame => {
                std::process::exit(0);
            }
            ButtonClicked::PauseGame => {
                next_state.set(GameState::Paused);
            }
            ButtonClicked::ResumeGame => {
                next_state.set(GameState::Playing);
            }
            ButtonClicked::RestartGame => {
                for game_over_menu in game_over_menu.iter() {
                    commands.entity(game_over_menu).despawn_recursive();
                }

                next_state.set(GameState::Start);
            }
        }
    }
}

fn update_score_text(
    mut text_query: Query<&mut Text, With<ScoreText>>,
    player_query: Query<&Player>,
) {
    for player in player_query.iter() {
        for mut text in text_query.iter_mut() {
            text.0 = format!("Stars: {}", player.stars);
        }
    }
}
