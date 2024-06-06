use glam::{IVec2, IVec3, Vec2, Vec3};

pub trait RoundableToIVec3 {
    fn round_to_ivec(&self) -> IVec3;
    fn ceil_to_ivec(&self) -> IVec3;
    fn floor_to_ivec(&self) -> IVec3;
}

pub trait RoundableToIVec2 {
    fn round_to_ivec(&self) -> IVec2;
    fn ceil_to_ivec(&self) -> IVec2;
    fn floor_to_ivec(&self) -> IVec2;
}

impl RoundableToIVec3 for Vec3 {
    fn round_to_ivec(&self) -> IVec3 {
        let v = self.round();
        IVec3{
            x: v.x as i32,
            y: v.y as i32,
            z: v.z as i32,
        }
    }

    fn ceil_to_ivec(&self) -> IVec3 {
        let v = self.ceil();
        IVec3{
            x: v.x as i32,
            y: v.y as i32,
            z: v.z as i32,
        }
    }

    fn floor_to_ivec(&self) -> IVec3 {
        let v = self.floor();
        IVec3{
            x: v.x as i32,
            y: v.y as i32,
            z: v.z as i32,
        }
    }
}

impl RoundableToIVec2 for Vec2 {
    fn round_to_ivec(&self) -> IVec2 {
        let v = self.round();
        IVec2{
            x: v.x as i32,
            y: v.y as i32,
        }
    }

    fn ceil_to_ivec(&self) -> IVec2 {
        let v = self.ceil();
        IVec2{
            x: v.x as i32,
            y: v.y as i32,
        }
    }

    fn floor_to_ivec(&self) -> IVec2 {
        let v = self.floor();
        IVec2{
            x: v.x as i32,
            y: v.y as i32,
        }
    }
}