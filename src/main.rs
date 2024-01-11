use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

mod utils;

use crate::utils::{
    buoyant_force, cross_section_area, damping, displaced_liquid_volume,
    off_center_cross_section_area, SPHERE_DRAG_COEFFICIENT,
};

#[derive(Component)]
struct Buoy;

fn setup(mut commands: Commands) {
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(20., 25., 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    let z_rotation = 0.1;
    // let z_rotation = 0.;

    // Raft
    let spawn_at = Vec3::new(0., 10., 0.);
    let raft_entity = commands
        .spawn((
            TransformBundle::from_transform(Transform::from_translation(spawn_at).with_rotation(Quat::from_rotation_z(z_rotation))),
            RigidBody::Dynamic,
        ))
        .with_children(|child_builder| {
            child_builder.spawn((
                TransformBundle::default(),
                Collider::cuboid(1.4, 0.6, 1.4),
                ColliderDensity(1.),
            ));
        })
        .id();

    // Buoys
    const DEFAULT_RADIUS: f32 = 0.5;
    const HALF_SIZE: f32 = 1.;
    let buoy_configs = [
        Vec3::new(-HALF_SIZE, 0., HALF_SIZE),
        Vec3::new(HALF_SIZE, 0., HALF_SIZE),
        Vec3::new(-HALF_SIZE, 0., -HALF_SIZE),
        Vec3::new(HALF_SIZE, 0., -HALF_SIZE),
    ];

    for position in buoy_configs {
        let buoy_entity = commands
            .spawn((
                TransformBundle::from_transform(Transform::from_translation(spawn_at + position)),
                Collider::ball(0.5),
                RigidBody::Dynamic,
                LinearDamping::default(),
                AngularDamping::default(),
                ExternalForce::default(),
                ColliderDensity(0.2),
                CollisionLayers::none(),
                Buoy,
            ))
            .id();

        commands.spawn(FixedJoint::new(raft_entity, buoy_entity).with_local_anchor_1(position));
    }
}

fn float(
    mut buoy_query: Query<
        (
            &GlobalTransform,
            &LinearVelocity,
            &Collider,
            &mut ExternalForce,
            &mut LinearDamping,
            &mut AngularDamping,
        ),
        With<Buoy>,
    >,
    time: Res<Time>,
) {
    let elapsed_time = time.elapsed().as_secs_f32();

    for (
        buoy_global_transform,
        linear_velocity,
        collider,
        mut external_force,
        mut linear_damping,
        mut angular_damping,
    ) in &mut buoy_query
    {
        let translation = buoy_global_transform.translation();
        let water_height = (elapsed_time + translation.x).sin();
        let radius = collider.shape().as_ball().unwrap().radius;
        let displaced_liquid_volume = displaced_liquid_volume(radius, translation.y, water_height);
        let buoyant_force = buoyant_force(displaced_liquid_volume);
        let submerged = (translation.y - water_height) - radius;

        let damping_coefficient;
        if submerged >= 0. {
            // Not submerged
            damping_coefficient = 0.;
        } else if submerged < -radius {
            // At least half submerged
            damping_coefficient = damping(
                linear_velocity.length(),
                cross_section_area(radius),
                SPHERE_DRAG_COEFFICIENT,
            );
        } else {
            // Less than half submerged
            damping_coefficient = damping(
                linear_velocity.length(),
                off_center_cross_section_area(radius, radius + submerged),
                SPHERE_DRAG_COEFFICIENT,
            );
        }

        external_force.set_force(buoyant_force);
        linear_damping.0 = damping_coefficient;
        angular_damping.0 = damping_coefficient;
    }
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(PhysicsPlugins::default());
    app.add_plugins(PhysicsDebugPlugin::default());

    app.add_systems(Startup, setup);
    app.add_systems(Update, float);

    app.run();
}
