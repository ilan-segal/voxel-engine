use bevy::prelude::*;

/// Axis-aligned bounding box
#[derive(Component)]
pub struct Aabb {
    pub x: f32,
    pub neg_x: f32,
    pub y: f32,
    pub neg_y: f32,
    pub z: f32,
    pub neg_z: f32,
}

impl Aabb {
    pub fn square_prism(width: f32, height: f32, eyline_height: f32) -> Self {
        let half_width = 0.5 * width;
        Self {
            x: half_width,
            neg_x: half_width,
            z: half_width,
            neg_z: half_width,
            y: height - eyline_height,
            neg_y: eyline_height,
        }
    }

    pub fn cube(side_length: f32) -> Self {
        let half = side_length * 0.5;
        Self {
            x: half,
            y: half,
            z: half,
            neg_x: half,
            neg_y: half,
            neg_z: half,
        }
    }

    pub fn get_dimensions(&self) -> Vec3 {
        Vec3 {
            x: self.x + self.neg_x,
            y: self.y + self.neg_y,
            z: self.z + self.neg_z,
        }
    }

    pub fn get_centre_offset(&self) -> Vec3 {
        let dimensions = self.get_dimensions();
        Vec3::new(self.neg_x, self.neg_y, self.neg_z) - dimensions * 0.5
    }

    pub fn with_scale(&self, scale: Vec3) -> Self {
        Self {
            x: self.x * scale.x,
            neg_x: self.neg_x * scale.x,
            y: self.y * scale.x,
            neg_y: self.neg_y * scale.x,
            z: self.z * scale.x,
            neg_z: self.neg_z * scale.x,
        }
    }
}
