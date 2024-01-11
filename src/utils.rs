use bevy::math::Vec3;
use std::f32::consts::PI;

pub const LIQUID_DENSITY: f32 = 1.025;
const GRAVITY: f32 = 9.807;
pub const SPHERE_DRAG_COEFFICIENT: f32 = 0.47;

pub fn damping(relative_velocity: f32, reference_area: f32, drag_coefficient: f32) -> f32 {
    0.5 * LIQUID_DENSITY * relative_velocity.powi(2) * reference_area * drag_coefficient
}

// https://www.omnicalculator.com/physics/buoyancy
pub fn buoyant_force(displaced_liquid_volume: f32) -> Vec3 {
    Vec3::Y * LIQUID_DENSITY * displaced_liquid_volume * GRAVITY
}

pub fn volume(radius: f32) -> f32 {
    4. / 3. * PI * radius.powi(3)
}

pub fn partial_volume(radius: f32, height: f32) -> f32 {
    PI / 3. * (3. * height.powi(2) * radius - height.powi(3))
}

pub fn displaced_liquid_volume(radius: f32, vertical_position: f32, water_height: f32) -> f32 {
    // Above surface
    if vertical_position >= water_height + radius {
        return 0.;
    }

    if vertical_position <= water_height - radius {
        // Sphere volume
        return volume(radius);
    }

    // Partially submerged
    partial_volume(radius, water_height - vertical_position + radius)
}
