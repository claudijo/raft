use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

mod utils;

use crate::utils::{
    buoyant_force, cross_section_area, damping, displaced_liquid_volume,
    off_center_cross_section_area, SPHERE_DRAG_COEFFICIENT,
};

#[derive(Component)]
struct Buoy;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(20., 10., 45.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_x(-40_f32.to_radians())),
        ..default()
    });

    // water
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Circle::new(20.0).into()),
        material: materials.add(Color::TURQUOISE.into()),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });

    // No initial angular velocity will make the first contact with water be quite explosive, and subsequent contacts
    // with water calm.
    // Initial angular velocity will make contact with water calm.
    // let z_rotation = 0.1;
    let z_rotation = 0.;

    // Raft
    let spawn_at = Vec3::new(0., 5., 0.);
    let raft_entity = commands
        .spawn((
            TransformBundle::from_transform(
                Transform::from_translation(spawn_at)
                    .with_rotation(Quat::from_rotation_z(z_rotation)),
            ),
            RigidBody::Dynamic,
        ))
        .with_children(|child_builder| {
            child_builder.spawn((
                TransformBundle::default(),
                Collider::cuboid(1.4, 0.6, 1.4),
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
            &Collider,
            &mut ExternalForce,
        ),
        With<Buoy>,
    >,
) {
    for (
        buoy_global_transform,
        collider,
        mut external_force,
    ) in &mut buoy_query
    {
        let translation = buoy_global_transform.translation();
        let water_height = 0.; //(elapsed_time + translation.x).sin();
        let radius = collider.shape().as_ball().unwrap().radius;
        let displaced_liquid_volume = displaced_liquid_volume(radius, translation.y, water_height);
        let buoyant_force = buoyant_force(displaced_liquid_volume);

        external_force.set_force(buoyant_force);
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
