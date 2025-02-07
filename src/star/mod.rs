use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::GameState;

// EVENTS -------------------------------------

// COMPONENTS -------------------------------------

#[derive(Component)]
pub struct Star;

// SYSTEMS -------------------------------------

fn despawn_stars(mut commands: Commands, stars: Query<Entity, With<Star>>) {
    for star in stars.iter() {
        commands.entity(star).despawn_recursive();
    }
}

fn spawn_star(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut timer: Local<f32>,
    // Query for any entity with both Transform and Collider components.
    colliders: Query<&Transform, With<Star>>,
    windows: Query<&Window>,
) {
    *timer += time.delta_secs();
    // Spawn a new star every 2 seconds if there are less than 4 stars.
    if *timer >= 2.0 {
        *timer = 0.0;
        let window = windows.single();
        let model = asset_server.load("star.png");

        const STAR_RADIUS: f32 = 16.0;
        const MAX_ATTEMPTS: usize = 10;

        // Closure to generate a random candidate position within window bounds,
        // ensuring the entire 32x32 star is visible and avoiding the top 56 pixels.
        let generate_candidate = || -> Vec2 {
            let half_star = 16.0;
            let margin_top = 56.0;
            let x_min = -window.width() / 2.0 + half_star;
            let x_max = window.width() / 2.0 - half_star;
            // Ensure the star is fully visible and not in the top 56 pixels:
            let y_min = -window.height() / 2.0 + half_star;
            let y_max = window.height() / 2.0 - half_star - margin_top;
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
                if other_position.distance(spawn_position) < STAR_RADIUS {
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

        // If no valid spawn point is found, do not spawn the star this cycle.
        if !valid_spawn {
            return;
        }

        commands.spawn((
            Sprite::from_image(model),
            Star,
            RigidBody::Fixed,
            Transform::from_translation(Vec3::new(spawn_position.x, spawn_position.y, 1.0)),
        ));
    }
}

fn start_pulse(time: Res<Time>, mut query: Query<&mut Transform, With<Star>>) {
    for mut transform in query.iter_mut() {
        let scale = 1.0 + (time.elapsed_secs() * 2.0).sin() * 0.1;
        transform.scale = Vec3::splat(scale);
    }
}
// PLUGIN -------------------------------------
pub struct StarPlugin;

impl Plugin for StarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Start), despawn_stars)
            .add_systems(
                FixedUpdate,
                (spawn_star, start_pulse)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
