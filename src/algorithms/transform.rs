use glam::{Vec3, Quat, Mat4};

#[derive(Debug, Clone)]
pub struct Transform {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
    matrix: Mat4,
    inverse_matrix: Mat4,
}

impl Default for Transform {
    fn default() -> Self {
        Transform::IDENTITY
    }
}

impl Transform {
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Transform {
        let mut result = Transform{position, 
            rotation, 
            scale, 
            matrix: Mat4::IDENTITY, 
            inverse_matrix: Mat4::IDENTITY};
        result.recalculate_matrix();
        result
    }

    pub const IDENTITY: Transform = Transform {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
        matrix: Mat4::IDENTITY,
        inverse_matrix: Mat4::IDENTITY
    };

    fn recalculate_matrix(&mut self) {
        self.matrix = Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position);
        self.inverse_matrix = self.matrix.inverse();
    } 

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    pub fn scale(&self) -> Vec3 {
        self.scale
    }

    pub fn set_position(&mut self, translation: Vec3) {
        self.position = translation;
        self.recalculate_matrix();
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        self.recalculate_matrix();
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
        self.recalculate_matrix();
    }

    pub fn set_position_and_rotation(&mut self, translation: Vec3, rotation: Quat) {
        self.position = translation;
        self.rotation = rotation;
        self.recalculate_matrix();
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    pub fn matrix(&self) -> Mat4 {
        self.matrix
    }

    pub fn inverse_matrix(&self) -> Option<Mat4> {
        if self.inverse_matrix.is_nan() {
            None
        }
        else {
            Some(self.inverse_matrix)
        }
    }
}