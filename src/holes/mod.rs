use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
    render::view::RenderLayers,
};
use bevy_audio_controller::prelude::{AudioFiles, GlobalPlayEvent};
use bevy_rapier2d::prelude::*;

use crate::{
    player::{self, Missile, Player},
    star::Star,
    GameState,
};

// Components -----------------------------------------------------------------
#[derive(Component)]
pub struct Hole {
    pub speed: f32,
}

impl Default for Hole {
    fn default() -> Self {
        Self { speed: 200.0 }
    }
}

#[derive(Component)]
struct HoleDirection {
    direction: Vec3,
}

// Systems --------------------------------------------------------------------
fn cleanup_holes(mut commands: Commands, holes: Query<Entity, With<Hole>>) {
    for hole in holes.iter() {
        commands.entity(hole).despawn_recursive();
    }
}

fn spawn_hole(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut timer: Local<f32>,
    // Query for any entity with both Transform and Collider components.
    colliders: Query<&Transform, With<Hole>>,
    windows: Query<&Window>,
    query: Query<Entity, With<Hole>>,
) {
    *timer += time.delta_secs();
    // Spawn a new hole every 3 seconds if there are less than 4 stars.
    let current_holes = query.iter().count();
    if *timer >= 3.0 && current_holes <= 8 {
        *timer = 0.0;
        let window = windows.single();
        let model = asset_server.load("hole_large_end.png");

        const HOLE_RADIUS: f32 = 40.0;
        const MAX_ATTEMPTS: usize = 10;

        // Closure to generate a random candidate position within window bounds,
        // ensuring the entire 32x32 hole is visible and avoiding the top 56 pixels.
        let generate_candidate = || -> Vec2 {
            let half_hole = HOLE_RADIUS / 2.;
            let margin_top = 56.0;
            let x_min = -window.width() / 2.0 + half_hole;
            let x_max = window.width() / 2.0 - half_hole;
            // Ensure the hole is fully visible and not in the top 56 pixels:
            let y_min = -window.height() / 2.0 + half_hole;
            let y_max = window.height() / 2.0 - half_hole - margin_top;
            let x = rand::random::<f32>() * (x_max - x_min) + x_min;
            let y = rand::random::<f32>() * (y_max - y_min) + y_min;
            Vec2::new(x, y)
        };

        // Attempt to find a non-overlapping spawn point.
        let mut spawn_position = generate_candidate();
        let mut valid_spawn = false;
        for _ in 0..MAX_ATTEMPTS {
            valid_spawn = true;
            // Check collision against all entities with a Collider.
            for transform in colliders.iter() {
                // Here we assume that the collider is a circle (Collider::ball),
                // so we perform a simple distance check. If your game uses different
                // collider shapes, you'll need to adjust this collision test accordingly.

                let other_position = transform.translation.truncate();
                if other_position.distance(spawn_position) < HOLE_RADIUS * 2.0 {
                    valid_spawn = false;
                    break;
                }
            }
            if valid_spawn {
                break;
            } else {
                spawn_position = generate_candidate();
            }
        }

        // If no valid spawn point is found, do not spawn the hole this cycle.
        if !valid_spawn {
            return;
        }

        let angle = rand::random::<f32>() * std::f32::consts::TAU;
        let dir = Vec3::new(angle.cos(), angle.sin(), 0.0);
        commands.spawn((
            Sprite::from_image(model),
            Hole::default(),
            RigidBody::Fixed,
            Transform::from_translation(Vec3::new(spawn_position.x, spawn_position.y, 1.0)),
            HoleDirection { direction: dir },
            RenderLayers::layer(1),
        ));
    }
}

fn check_hole_star_collision(
    mut commands: Commands,
    holes: Query<&Transform, With<Hole>>,
    stars: Query<(Entity, &Transform), With<Star>>,
) {
    for hole_transform in holes.iter() {
        for (star_entity, star_transform) in stars.iter() {
            let distance = hole_transform
                .translation
                .distance(star_transform.translation);
            if distance < 42. {
                // Adjust collision radius as needed
                commands.entity(star_entity).despawn();
            }
        }
    }
}
fn check_hole_collisions(mut commands: Commands, query: Query<(Entity, &Transform), With<Hole>>) {
    let collision_distance = 80.0;
    let mut to_despawn = std::collections::HashSet::new();

    for [item_a, item_b] in query.iter_combinations() {
        let (entity_a, transform_a) = item_a;
        let (entity_b, transform_b) = item_b;
        let distance = transform_a.translation.distance(transform_b.translation);
        if distance < collision_distance {
            to_despawn.insert(entity_a);
            to_despawn.insert(entity_b);
        }
    }

    for entity in to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

fn move_holes(
    time: Res<Time>,
    windows: Query<&Window>,
    stars: Query<&Transform, (With<Star>, Without<Hole>, Without<Player>)>,
    mut holes: Query<(Entity, &mut Transform, &Hole), With<Hole>>,
    players: Query<(Entity, &mut Transform), (With<Player>, Without<Hole>, Without<Star>)>,
) {
    let window = windows.single();
    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;
    // The hole has a diameter of 80 (radius 40).
    const HOLE_RADIUS: f32 = 40.0;
    // Prevent holes from moving into the top 50 pixels.
    const TOP_MARGIN: f32 = 50.0;
    // Speed at which holes move.

    if stars.iter().count() == 0 {
        for (entity, mut transform, hole) in holes.iter_mut() {
            let hole_pos = transform.translation;
            let mut nearest_star: Option<Vec3> = None;
            let mut min_distance = f32::MAX;

            // Find the nearest player to this hole.
            for player_transform in players.iter() {
                let player_pos = player_transform.1.translation;
                let distance = hole_pos.distance(player_pos);
                if distance < min_distance {
                    min_distance = distance;
                    nearest_star = Some(player_pos);
                }
            }

            // If a player is found, move the hole toward it.
            if let Some(target_star) = nearest_star {
                // Calculate the normalized direction vector.
                let direction = (target_star - hole_pos).normalize();
                let displacement = direction * hole.speed * time.delta_secs();
                let target = transform.translation + displacement;

                // Clamp target to ensure the entire hole remains within visible bounds.
                let clamped_x = target
                    .x
                    .clamp(-half_width + HOLE_RADIUS, half_width - HOLE_RADIUS);
                let clamped_y = target.y.clamp(
                    -half_height + HOLE_RADIUS,
                    (half_height - TOP_MARGIN) - HOLE_RADIUS,
                );
                let final_target = Vec3::new(clamped_x, clamped_y, target.z);

                println!(
                    "Moving hole {:?}, towards player at {:?}, dest: {}",
                    entity.index(),
                    target_star,
                    final_target
                );
                transform.translation = final_target;
            }
        }
    }

    for (entity, mut transform, hole) in holes.iter_mut() {
        let hole_pos = transform.translation;
        let mut nearest_star: Option<Vec3> = None;
        let mut min_distance = f32::MAX;

        // Find the nearest star to this hole.
        for star_transform in stars.iter() {
            let star_pos = star_transform.translation;
            let distance = hole_pos.distance(star_pos);
            if distance < min_distance {
                min_distance = distance;
                nearest_star = Some(star_pos);
            }
        }

        // If a star is found, move the hole toward it.
        if let Some(target_star) = nearest_star {
            // Calculate the normalized direction vector.
            let direction = (target_star - hole_pos).normalize();
            let displacement = direction * hole.speed * time.delta_secs();
            let target = transform.translation + displacement;

            // Clamp target to ensure the entire hole remains within visible bounds.
            let clamped_x = target
                .x
                .clamp(-half_width + HOLE_RADIUS, half_width - HOLE_RADIUS);
            let clamped_y = target.y.clamp(
                -half_height + HOLE_RADIUS,
                (half_height - TOP_MARGIN) - HOLE_RADIUS,
            );
            let final_target = Vec3::new(clamped_x, clamped_y, target.z);

            println!(
                "Moving hole {:?}, towards star at {:?}, dest: {}",
                entity.index(),
                target_star,
                final_target
            );
            transform.translation = final_target;
        }
    }
}

fn check_missile_hit(
    mut commands: Commands,
    mut missiles: Query<(Entity, &Transform), With<Missile>>,
    holes: Query<(Entity, &Transform), (With<Hole>, Without<Missile>)>,
    mut sfx_play_ew: EventWriter<GlobalPlayEvent>,
) {
    //check if missile hits hole
    for (missile_entity, missile_transform) in missiles.iter_mut() {
        for (hole_enityt, hole_transform) in holes.iter() {
            let distance = missile_transform
                .translation
                .distance(hole_transform.translation);
            if distance < 40. {
                commands.entity(missile_entity).despawn();
                commands.entity(hole_enityt).despawn();
                let event = GlobalPlayEvent::new(AudioFiles::ExplosionWAV).with_settings(
                    PlaybackSettings {
                        mode: PlaybackMode::Once,
                        volume: Volume::new(10.),
                        ..Default::default()
                    },
                );
                sfx_play_ew.send(event);
            }
        }
    }
}

// Plugin ---------------------------------------------------------------------

pub struct HolePlugin;

impl Plugin for HolePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Start), cleanup_holes)
            .add_systems(
                FixedUpdate,
                (
                    spawn_hole,
                    check_hole_star_collision,
                    check_hole_collisions,
                    move_holes,
                    check_missile_hit,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
