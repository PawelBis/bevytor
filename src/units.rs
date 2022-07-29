use super::mouse_input::MoveCommandEvent;
use bevy::core::FixedTimestep;
use bevy::math;
use bevy::prelude::*;
use std::thread::current;

const MOVEMENT_TIME_STEP: f64 = 1.0 / 120.0;

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_unit_system)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(MOVEMENT_TIME_STEP))
                    .with_system(path_movement_system),
            )
            .add_system(path_update_system);
        //.add_system(path_movement_system);
    }
}

#[derive(Component)]
struct UnitSpawner;

#[derive(Component)]
struct Unit;

#[derive(Component)]
struct Velocity {
    value: f32,
}

#[derive(Component, Default)]
struct Movement {
    path: Vec<Vec3>,
}

fn spawn_unit_system(
    mut commands: Commands,
    _query: Query<(&UnitSpawner, &Transform), With<UnitSpawner>>,
) {
    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(40.0, 40.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Unit)
        .insert(Movement::default())
        .insert(Velocity { value: 1000.0 });
}

fn path_update_system(
    mut mouse_events: EventReader<MoveCommandEvent>,
    mut movement_query: Query<(&mut Movement, &Velocity, &Transform)>,
) {
    for event in mouse_events.iter() {
        let destination = event.destination.extend(0.0);
        for (mut movement, velocity, transform) in movement_query.iter_mut() {
            // Skip path calculation if the location almost equal to current position
            if (transform.translation - destination).length_squared()
                < velocity.value * MOVEMENT_TIME_STEP as f32
            {
                continue;
            }
            if movement.path.is_empty() {
                movement.path = vec![destination];
            } else {
                movement.path[0] = destination;
            }
        }
    }
}

fn path_movement_system(
    mut units_query: Query<(&mut Movement, &mut Transform, &Velocity), With<Unit>>,
) {
    for (mut movement, mut transform, velocity) in units_query.iter_mut() {
        if movement.path.is_empty() {
            continue;
        }
        let current_location = transform.translation;
        let destination = movement.path[0];
        let direction_to_destination = (destination - current_location).normalize();
        let delta_movement = direction_to_destination * velocity.value * MOVEMENT_TIME_STEP as f32;
        let final_location = current_location + delta_movement;

        // Clear path point and snap to destination if the distance is too short
        if (final_location - destination).length_squared() < delta_movement.length_squared() {
            transform.translation = destination;
            movement.path.pop();
            continue;
        }
        transform.translation = final_location;
    }
}
