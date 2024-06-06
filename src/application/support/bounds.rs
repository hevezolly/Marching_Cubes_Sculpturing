use std::{cmp::{max, min}, marker::PhantomData, ops::{Add, Div, Mul, Sub}, process::Output};

use glam::{ivec3, vec3, IVec3, Vec3};
use num::Zero;

#[derive(Debug, Clone, Copy)]
pub struct Bounds<T: Clone + Copy> {
    min_max: Option<(T, T)>,
}

pub fn part_min<'a, T: PartialOrd>(v1: &'a T, v2: &'a T) -> &'a T {
    if v1 < v2 {v1} else {v2}
}

pub fn part_max<'a, T: PartialOrd>(v1: &'a T, v2: &'a T) -> &'a T {
    if v1 > v2 {v1} else {v2}
}

pub trait Cord3D {
    type Value;
    fn x(&self) -> Self::Value;
    fn y(&self) -> Self::Value;
    fn z(&self) -> Self::Value;

    fn new(x: Self::Value, y: Self::Value, z: Self::Value) -> Self;
}

pub trait CordOps {
    fn max(v1: &Self, v2: &Self) -> Self;
    fn min(v1: &Self, v2: &Self) -> Self;
    fn contains(val: &Self, min: &Self, max: &Self) -> bool;
}

impl<T: Cord3D> CordOps for T 
    where <T as Cord3D>::Value: PartialOrd + Clone + Copy
{
    fn max(v1: &Self, v2: &Self) -> Self {
        T::new(part_max(&v1.x(), &v2.x()).clone(), 
                part_max(&v1.y(), &v2.y()).clone(),
                part_max(&v1.z(), &v2.z()).clone())
    }

    fn min(v1: &Self, v2: &Self) -> Self {
        T::new(part_min(&v1.x(), &v2.x()).clone(), 
                part_min(&v1.y(), &v2.y()).clone(),
                part_min(&v1.z(), &v2.z()).clone())
    }

    fn contains(val: &Self, min: &Self, max: &Self) -> bool {
        val.x() >= min.x() && val.x() <= max.x() && 
        val.y() >= min.y() && val.y() <= max.y() &&
        val.z() >= min.z() && val.z() <= max.z()
    }
}

impl Cord3D for Vec3 {
    type Value = f32;

    fn x(&self) -> Self::Value {
        self.x
    }

    fn y(&self) -> Self::Value {
        self.y
    }

    fn z(&self) -> Self::Value {
        self.z
    }
    
    fn new(x: Self::Value, y: Self::Value, z: Self::Value) -> Self {
        vec3(x, y, z)
    }
}

impl Cord3D for IVec3 {
    type Value = i32;

    fn x(&self) -> Self::Value {
        self.x
    }

    fn y(&self) -> Self::Value {
        self.y
    }

    fn z(&self) -> Self::Value {
        self.z
    }

    fn new(x: Self::Value, y: Self::Value, z: Self::Value) -> Self {
        ivec3(x, y, z)
    }
}

impl<T: Cord3D + CordOps + Clone + Copy> Bounds<T> {
    pub fn empty() -> Bounds<T> {
        Bounds { min_max: None }
    }

    pub fn encapsulate(&mut self, vec: T) {
        match &self.min_max {
            Some((mi, ma)) => {
                let new_min = T::min(&mi, &vec);
                let new_max = T::max(&ma, &vec);
                self.min_max = Some((new_min, new_max));
            },
            None => self.min_max = Some((vec.clone(), vec)),
        }
    }

    pub fn encapsulate_other(&mut self, other: &Bounds<T>) {
        match &other.min_max {
            Some((mi, ma)) => { 
                self.encapsulate(mi.clone());
                self.encapsulate(ma.clone());
            },
            None => ()
        }
    }

    pub fn new(v: T) -> Bounds<T> {
        Bounds { min_max: Some((v.clone(), v)) }
    }
    
    pub fn min_max(v1: T, v2: T) -> Bounds<T> {
        let mut b = Bounds::new(v1);
        b.encapsulate(v2);
        b
    }

    pub fn has_values(&self) -> bool {self.min_max.is_some()}

    pub fn contains(&self, cord: T) -> bool {
        match &self.min_max {
            Some((mi, ma)) => T::contains(&cord, &mi, &ma),
            None => false,
        }
    }
}

impl<T: Cord3D + CordOps + Clone + Copy> Bounds<T> 
    where <T as Cord3D>::Value: Zero
{
    pub fn min(&self) -> T {
        match &self.min_max {
            Some((min, _)) => min.clone(),
            None => T::new(T::Value::zero(), T::Value::zero(), T::Value::zero()),
        }
    }

    pub fn max(&self) -> T {
        match &self.min_max {
            Some((_, max)) => max.clone(),
            None => T::new(T::Value::zero(), T::Value::zero(), T::Value::zero()),
        }
    }
}

impl<T: Cord3D + CordOps + Copy + Clone + Add<Output = T>> Bounds<T> {
    pub fn offset(&self, offset: T) -> Bounds<T> {
        match self.min_max {
            Some((mi, ma)) => Bounds::min_max(mi + offset, ma + offset),
            None => Bounds::empty(),
        }
    }
}

impl<T: Cord3D + CordOps + Copy + Clone + Sub<Output = T>> Bounds<T>
    where <T as Cord3D>::Value: Zero 
{
    pub fn size(&self) -> T {
        match self.min_max {
            Some((mi, ma)) => ma - mi,
            None => T::new(T::Value::zero(), T::Value::zero(), T::Value::zero()),
        }
    }
}

impl<T: Cord3D + CordOps + Copy + Clone + Mul<Output = T>> Bounds<T> {
    pub fn scale(&self, scale_factor: T) -> Bounds<T> {
        match self.min_max {
            Some((mi, ma)) => Bounds::min_max(mi * scale_factor, ma * scale_factor),
            None => Bounds::empty(),
        }
    }
}

impl Bounds<Vec3> {
    pub fn iterate_cords(&self) -> BoundsIterator {
        if let Some((min, max)) = self.min_max {
            let min = min.floor();
            let min = ivec3(min.x as i32, min.y as i32, min.z as i32);
            let max = max.floor();
            let max = ivec3(max.x as i32, max.y as i32, max.z as i32);
            BoundsIterator { current: min, min, max: max }
        }
        else {
            BoundsIterator { current: ivec3(1, 1, 1), min: IVec3::ZERO, max: IVec3::ZERO }
        }
    }
}

impl Bounds<IVec3> {
    pub fn iterate_cords(&self) -> BoundsIterator {
        if let Some((min, max)) = self.min_max {
            BoundsIterator { current: min, min, max: max - IVec3::ONE }
        }
        else {
            BoundsIterator { current: ivec3(1, 1, 1), min: IVec3::ZERO, max: IVec3::ZERO }
        }
    }
}

pub struct BoundsIterator {
    current: IVec3,
    min: IVec3,
    max: IVec3,
}

impl Iterator for BoundsIterator {
    type Item = IVec3;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.x > self.max.x {return None;}

        let result = self.current;

        if self.current.y > self.max.y && self.current.z > self.max.z {
            self.current.x += 1;
            self.current.y = self.min.y;
            self.current.z = self.min.z;
        }
        else if self.current.z > self.max.z {
            self.current.y += 1;
            self.current.z = self.min.z;
        }
        else {
            self.current.z += 1;
        };

        Some(result)
    }
}