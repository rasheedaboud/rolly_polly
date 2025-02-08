use bevy::prelude::*;
use bevy_audio_controller::prelude::{AudioFiles, GlobalPlayEvent};
use bevy_rapier2d::prelude::*;

use crate::{holes::Hole, star::Star, GameState, Health};

/// EVENTS -------------------------------------
#[derive(Event)]
pub struct PlayerCollidedHole;
#[derive(Event)]
pub struct PlayerCollectedStar;

#[derive(Event)]
pub struct PlayerAddLife;

#[derive(Event)]
pub struct PlayerAddMissiles;

/// COMPONENTS -------------------------------------
#[derive(Component, Copy, Clone)]
pub struct Player {
    pub life: i8,
    pub stars: u32,
    pub speed: f32,
    pub missiles: f32,
}

#[derive(Component)]
pub struct Missile;

impl Default for Player {
    fn default() -> Self {
        Player {
            life: 3,
            stars: 0,
            speed: 750.0,
            missiles: 500.0,
        }
    }
}

// SYSTEMS -------------------------------------

fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player: Query<Entity, With<Player>>,
) {
    for player in player.iter() {
        commands.entity(player).despawn_recursive();
    }

    commands.spawn((
        Sprite::from_image(asset_server.load("ball_blue_large.png")),
        Transform::from_xyz(0., 0., 1.),
        Player::default(),
        RigidBody::Dynamic,
        Collider::ball(32.0),
        GravityScale(0.),
        Velocity::zero(),
    ));
}

fn gamepad_movement(
    gamepads: Query<&Gamepad>,
    mut query: Query<(&mut Velocity, &Player, &mut Transform), With<Player>>,
    time: Res<Time>,
) {
    for (mut velocity, player, mut transform) in query.iter_mut() {
        for gamepad in &gamepads {
            let speed = player.speed;
            let mut direction = Vec2::ZERO;

            // Get stick input
            if let (Some(left_stick_x), Some(left_stick_y)) = (
                gamepad.get(GamepadAxis::LeftStickX),
                gamepad.get(GamepadAxis::LeftStickY),
            ) {
                direction = Vec2::new(left_stick_x, left_stick_y);

                // Only rotate if stick has significant movement
                if direction.length() > 0.1 {
                    let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
                    transform.rotation = Quat::from_rotation_z(angle);
                }
            }

            // Normalize direction for movement
            if direction != Vec2::ZERO {
                direction = direction.normalize();
            }

            // Apply movement
            let target_velocity = direction * speed;
            let acceleration = 5.0;
            velocity.linvel = velocity
                .linvel
                .lerp(target_velocity, acceleration * time.delta_secs());
        }
    }
}

fn fire_missile_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    query: Single<(&mut Transform, &mut Player), With<Player>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut sfx_play_ew: EventWriter<GlobalPlayEvent>,
) {
    let transform = &query.0;
    let mut player = *query.1;
    // Determine missile firing parameters.
    // Use the player's current rotation to compute the firing direction.
    let missile_speed = 1000.0;

    // default fire direction to player direction from transform
    let forward = transform.rotation * Vec3::Y;
    let fire_direction = Vec2::new(forward.x, forward.y);

    if fire_direction != Vec2::ZERO && player.missiles > 0.0 && keys.just_pressed(KeyCode::Space) {
        // Offset the spawn position slightly so the missile doesn't overlap the player.
        let spawn_position = transform.translation + (fire_direction.extend(0.0) * 50.0);
        let angle = fire_direction.y.atan2(fire_direction.x) - std::f32::consts::FRAC_PI_2;
        commands.spawn((
            Sprite::from_image(asset_server.load("ball_red_small.png")),
            Transform {
                translation: spawn_position,
                rotation: Quat::from_rotation_z(angle),
                ..Default::default()
            },
            RigidBody::Dynamic,
            GravityScale(0.),
            Velocity {
                linvel: fire_direction * missile_speed,
                angvel: 0.0,
            },
            Missile,
        ));
        player.missiles -= 1.0;
        let event = GlobalPlayEvent::new(AudioFiles::PopOGG).with_settings(PlaybackSettings::ONCE);
        sfx_play_ew.send(event);
    }
}

fn fire_missile(
    gamepads: Query<&Gamepad>,
    keys: Res<ButtonInput<KeyCode>>,
    query: Single<(&mut Transform, &mut Player), With<Player>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut sfx_play_ew: EventWriter<GlobalPlayEvent>,
) {
    for gamepad in &gamepads {
        let transform = &query.0;
        let mut player = *query.1;
        // Determine missile firing parameters.
        // Use the player's current rotation to compute the firing direction.
        let missile_speed = 1000.0;

        // default fire direction to player direction from transform
        let forward = transform.rotation * Vec3::Y;
        let mut fire_direction = Vec2::new(forward.x, forward.y);

        if let (Some(stick_x), Some(stick_y)) = (
            gamepad.get(GamepadAxis::LeftStickX),
            gamepad.get(GamepadAxis::LeftStickY),
        ) {
            let stick_direction = Vec2::new(stick_x, stick_y);
            if stick_direction.length() > 0.1 {
                fire_direction = stick_direction.normalize();
            }
        }

        if fire_direction != Vec2::ZERO
            && player.missiles > 0.0
            && (gamepad.just_pressed(GamepadButton::South) || keys.just_pressed(KeyCode::Space))
        {
            // Offset the spawn position slightly so the missile doesn't overlap the player.
            let spawn_position = transform.translation + (fire_direction.extend(0.0) * 50.0);
            let angle = fire_direction.y.atan2(fire_direction.x) - std::f32::consts::FRAC_PI_2;
            commands.spawn((
                Sprite::from_image(asset_server.load("ball_red_small.png")),
                Transform {
                    translation: spawn_position,
                    rotation: Quat::from_rotation_z(angle),
                    ..Default::default()
                },
                RigidBody::Dynamic,
                GravityScale(0.),
                Velocity {
                    linvel: fire_direction * missile_speed,
                    angvel: 0.0,
                },
                Missile,
            ));
            player.missiles -= 1.0;
            let event =
                GlobalPlayEvent::new(AudioFiles::PopOGG).with_settings(PlaybackSettings::ONCE);
            sfx_play_ew.send(event);
        }
    }
}

fn player_movement(
    mut query: Query<(&mut Velocity, &Player, &mut Transform), With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (mut velocity, player, mut transform) in query.iter_mut() {
        let speed = player.speed;

        let mut direction = Vec2::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }
        // Normalize so diagonal movement isn’t faster.
        if direction != Vec2::ZERO {
            direction = direction.normalize();
            let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
            transform.rotation = Quat::from_rotation_z(angle);
        }

        // Option 2: Smoothly interpolate toward the desired velocity (for gradual acceleration)
        let target_velocity = direction * speed;
        let acceleration = 5.0;
        velocity.linvel = velocity
            .linvel
            .lerp(target_velocity, acceleration * time.delta_secs());
    }
}

fn star_collision_event(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    stars: Query<(Entity, &Transform), With<Star>>,
    mut events: EventWriter<PlayerCollectedStar>,
) {
    for player_transform in player.iter() {
        for (star_entity, star_transform) in stars.iter() {
            let delta = player_transform.translation - star_transform.translation;
            // Player half size is 32 and star half size is 16
            if delta.x.abs() < 48.0 && delta.y.abs() < 48.0 {
                commands.entity(star_entity).despawn();
                events.send(PlayerCollectedStar);
            }
        }
    }
}

fn hole_collision_event(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    holes: Query<(Entity, &Transform), With<Hole>>,
    mut events: EventWriter<PlayerCollidedHole>,
) {
    for player_transform in player.iter() {
        for (hole_entity, hole_transform) in holes.iter() {
            let delta = player_transform.translation - hole_transform.translation;
            //PLayer half size is 32 and hole half size is 40
            if delta.x.abs() < 48.0 && delta.y.abs() < 48.0 {
                // Adjust collision radius as needed
                commands.entity(hole_entity).despawn();
                events.send(PlayerCollidedHole);
            }
        }
    }
}
fn handle_star_collection(
    mut player: Query<&mut Player>,
    mut events: EventReader<PlayerCollectedStar>,
    mut sfx_play_ew: EventWriter<GlobalPlayEvent>,
    mut life_event: EventWriter<PlayerAddLife>,
    mut missile_event: EventWriter<PlayerAddMissiles>,
) {
    for _ in events.read() {
        for mut player in player.iter_mut() {
            player.stars += 1;
            let event =
                GlobalPlayEvent::new(AudioFiles::PowerUp2OGG).with_settings(PlaybackSettings::ONCE);
            sfx_play_ew.send(event);
            if player.stars % 10 == 0 {
                player.speed = player.speed * 1.1;
                life_event.send(PlayerAddLife);
                missile_event.send(PlayerAddMissiles);
            }
        }
    }
}

fn handle_player_collided_with_hole_event(
    mut player: Query<(&mut Player, &mut Transform), With<Player>>,
    mut events: EventReader<PlayerCollidedHole>,
) {
    for _ in events.read() {
        for (mut player, mut transform) in player.iter_mut() {
            player.life -= 1;
            transform.translation = Vec3::new(0., 0., 0.);
        }
    }
}

fn handle_player_add_life(
    mut players: Query<&mut Player>,
    mut events: EventReader<PlayerAddLife>,
    health: Query<(Entity, &Transform), With<Health>>,
    window: Query<&Window>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for _ in events.read() {
        for mut player in players.iter_mut() {
            player.life += 1;
            println!("Player Lives: {}", player.life);
            let window = window.single();
            for health in health.iter() {
                commands.entity(health.0).despawn();
            }

            // Spawn health indicators
            for i in 0..player.life {
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
        }
    }
}

fn handle_player_add_missiles(
    mut players: Query<&mut Player>,
    mut events: EventReader<PlayerAddMissiles>,
) {
    for _ in events.read() {
        for mut player in players.iter_mut() {
            player.missiles += 10.0;
        }
    }
}

fn handle_player_update_health_event(
    player: Query<&Player>,
    health: Query<(Entity, &Transform), With<Health>>,
    mut events: EventReader<PlayerCollidedHole>,
    window: Query<&Window>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
    mut sfx_play_ew: EventWriter<GlobalPlayEvent>,
) {
    for _ in events.read() {
        for player in player.iter() {
            if player.life == 0 {
                next_state.set(GameState::GameOver);
            } else {
                let event = GlobalPlayEvent::new(AudioFiles::ExplosionCrunch001OGG)
                    .with_settings(PlaybackSettings::ONCE);
                sfx_play_ew.send(event);
                let window = window.single();

                for health in health.iter() {
                    commands.entity(health.0).despawn();
                }

                // Spawn health indicators
                for i in 0..player.life {
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
            }
        }
    }
}

// PLUGIN -------------------------------------
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Start), setup_player)
            .add_systems(
                FixedUpdate,
                (
                    gamepad_movement,
                    fire_missile_keyboard,
                    fire_missile,
                    player_movement,
                    handle_star_collection,
                    hole_collision_event,
                    star_collision_event,
                    handle_player_collided_with_hole_event,
                    handle_player_update_health_event,
                    handle_player_add_life,
                    handle_player_add_missiles,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_event::<PlayerCollectedStar>()
            .add_event::<PlayerCollidedHole>()
            .add_event::<PlayerAddMissiles>()
            .add_event::<PlayerAddLife>();
    }
}
