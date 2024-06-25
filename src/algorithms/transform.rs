use glam::{Vec3, Quat, Mat4};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    position: Vec3,
    rotation: Quat,
    scale: Vec3
}

impl Default for Transform {
    fn default() -> Self {
        Transform::IDENTITY
    }
}

impl Transform {
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Transform {
        Transform{position, 
            rotation, 
            scale}
    }

    pub fn from_position(position: Vec3) -> Transform {
        Transform::new(position, Quat::IDENTITY, Vec3::ONE)
    }

    pub const IDENTITY: Transform = Transform {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE
    };

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
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
    }

    pub fn set_position_and_rotation(&mut self, translation: Vec3, rotation: Quat) {
        self.position = translation;
        self.rotation = rotation;
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
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    pub fn inverse_matrix(&self) -> Mat4 {
        self.matrix().inverse()
    }
}