use glam::Vec3;

pub mod camera;
pub mod raycast;
pub mod transform;
pub mod grid_line_intersection;

#[derive(Debug, Clone)]
pub struct Triangle([Vec3;3]);

impl Triangle {
    pub fn new(a: Vec3, b: Vec3, c: Vec3) -> Triangle {
        Triangle([a, b, c])
    }

    pub fn a(&self) -> Vec3 {
        self.0[0]
    }

    pub fn b(&self) -> Vec3 {
        self.0[1]
    }

    pub fn c(&self) -> Vec3 {
        self.0[2]
    }

    pub fn ab(&self) -> Vec3 {
        self.b() - self.a()
    }

    pub fn bc(&self) -> Vec3 {
        self.c() - self.b()
    }

    pub fn ca(&self) -> Vec3 {
        self.a() - self.c()
    }

    pub fn ac(&self) -> Vec3 {
        self.c() - self.a()
    }

    pub fn normal(&self) -> Vec3 {
        Vec3::cross(self.ab(), self.ac()).normalize()
    }
}
