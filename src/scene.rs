//! Angles are specified in degrees

use serde::{Deserialize, Serialize};

use crate::{fb::Color, material::Material, raytrace};

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Rotation {
    Euler {
        #[serde(default)]
        roll: f32,
        #[serde(default)]
        pitch: f32,
        #[serde(default)]
        yaw: f32,
    },
    Quaternion {
        x: f32,
        y: f32,
        z: f32,
        w: f32,
    },
}

impl Rotation {
    fn into_raytrace(self) -> na::UnitQuaternion<f32> {
        match self {
            Rotation::Euler { roll, pitch, yaw } => na::UnitQuaternion::from_euler_angles(
                pitch.to_radians(),
                yaw.to_radians(),
                roll.to_radians(),
            ),

            Rotation::Quaternion { x, y, z, w } => {
                na::UnitQuaternion::new_normalize(na::Quaternion::new(x, y, z, w))
            }
        }
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Rotation::Euler {
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct Translation {
    #[serde(default)]
    x: f32,
    #[serde(default)]
    y: f32,
    #[serde(default)]
    z: f32,
}

impl Translation {
    fn into_raytrace(self) -> na::Translation3<f32> {
        na::Translation3::new(self.x, self.y, self.z)
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct Position {
    #[serde(default)]
    trans: Translation,
    #[serde(default)]
    rot: Rotation,
}

impl Position {
    fn into_raytrace(self) -> na::Isometry3<f32> {
        na::Isometry3::from_parts(self.trans.into_raytrace(), self.rot.into_raytrace())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Shape {
    Ball(f32),
    Cuboid { x: f32, y: f32, z: f32 },
}

impl Shape {
    fn into_raytrace(self) -> nc::shape::ShapeHandle<f32> {
        match self {
            Shape::Ball(radius) => nc::shape::ShapeHandle::new(nc::shape::Ball::new(radius)),
            Shape::Cuboid { x, y, z } => {
                nc::shape::ShapeHandle::new(nc::shape::Cuboid::new(na::Vector3::new(x, y, z)))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Object {
    pos: Position,
    shape: Shape,
    #[serde(default)]
    mat: Material,
}

impl Object {
    fn into_raytrace(self) -> raytrace::RaytraceObject {
        raytrace::RaytraceObject {
            pos: self.pos.into_raytrace(),
            shape: self.shape.into_raytrace(),
            mat: self.mat,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct LightSource {
    pos: Position,
    brightness: Color,
    kind: LightSourceKind,
}

impl LightSource {
    fn into_raytrace(self) -> raytrace::LightSource {
        raytrace::LightSource {
            pos: self.pos.into_raytrace(),
            brightness: self.brightness,
            kind: match self.kind {
                LightSourceKind::Point => raytrace::LightSourceKind::Point,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum LightSourceKind {
    Point,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Scene {
    #[serde(default = "default_size")]
    pub size: (u16, u16),
    #[serde(default = "default_fov")]
    pub fov: f32,
    #[serde(default = "default_steps")]
    pub steps: usize,
    #[serde(default)]
    pub multisample: bool,
    #[serde(default)]
    camera: Position,
    objects: Vec<Object>,
    lights: Vec<LightSource>,
}

impl Scene {
    pub fn unpack(
        self,
    ) -> (
        na::Isometry3<f32>,
        Vec<raytrace::RaytraceObject>,
        Vec<raytrace::LightSource>,
    ) {
        (
            self.camera.into_raytrace(),
            self.objects
                .into_iter()
                .map(Object::into_raytrace)
                .collect(),
            self.lights
                .into_iter()
                .map(LightSource::into_raytrace)
                .collect(),
        )
    }
}

fn default_size() -> (u16, u16) {
    (640, 480)
}

fn default_fov() -> f32 {
    85.0
}

fn default_steps() -> usize {
    4
}
