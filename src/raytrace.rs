use na::{Isometry3, Point3, UnitQuaternion, Vector3};
use nc::{
    query::{Ray, RayIntersection},
    shape::ShapeHandle,
    pipeline::{
        object::{
            CollisionGroups,
            CollisionObject as CollisionObject_,
            GeometricQueryType
        },
        world::CollisionWorld as CollisionWorld_
    }
};

use crate::{
    fb::{Color, Fb},
    material::{Material, Phong, Reflect},
};

type CollisionWorld = CollisionWorld_<f32, WorldData>;
type CollisionObject = CollisionObject_<f32, WorldData>;

pub struct RaytraceObject {
    pub pos: Isometry3<f32>,
    pub shape: ShapeHandle<f32>,
    pub mat: Material,
}

impl RaytraceObject {
    fn unpack(self) -> (Isometry3<f32>, ShapeHandle<f32>, WorldData) {
        (self.pos, self.shape, WorldData { mat: self.mat })
    }
}

struct WorldData {
    mat: Material,
}

pub struct LightSource {
    pub pos: Isometry3<f32>,
    pub brightness: Color,
    pub kind: LightSourceKind,
}

impl LightSource {
    pub fn point(brightness: Color, pos: Point3<f32>) -> Self {
        LightSource {
            pos: Isometry3::from_parts(
                na::Translation3::new(pos.x, pos.y, pos.z),
                na::UnitQuaternion::identity(),
            ),
            brightness,
            kind: LightSourceKind::Point,
        }
    }
}

pub enum LightSourceKind {
    Point,
}

struct RtConfig {
    ambient: Color,
    world: CollisionWorld,
    lights: Vec<LightSource>,
}

struct RayData {
    ray: Ray<f32>,
    steps_left: usize,
    refraction_stack: rpds::Stack<f32>,
}

impl RayData {
    fn refract(&self, ray: Ray<f32>, index: f32) -> Self {
        RayData {
            ray,
            steps_left: self.steps_left - 1,
            refraction_stack: self.refraction_stack.push(index),
        }
    }

    fn unrefract(&self, ray: Ray<f32>) -> Self {
        RayData {
            ray,
            steps_left: self.steps_left - 1,
            refraction_stack: self.refraction_stack.pop().expect("Refraction stack empty"),
        }
    }

    fn push(&self, ray: Ray<f32>) -> Self {
        RayData {
            ray,
            steps_left: self.steps_left - 1,
            refraction_stack: self.refraction_stack.clone(),
        }
    }
}

pub fn raytrace(
    size: (u16, u16),
    fov: f32,
    steps: usize,
    camera: Isometry3<f32>,
    objects: Vec<RaytraceObject>,
    lights: Vec<LightSource>,
) -> Fb {
    let fov = fov.to_radians();

    let mut world = CollisionWorld::new(0.0);
    for obj in objects {
        let (pos, shape, data) = obj.unpack();
        world.add(
            pos,
            shape,
            CollisionGroups::new(),
            GeometricQueryType::Contacts(0.0, 0.0),
            data,
        );
    }
    world.update();

    let plane_distance = (fov / 2.0).tan().recip();

    let config = RtConfig {
        ambient: Color::new(0.0, 0.0, 0.1),
        world,
        lights,
    };

    let func = |x, y| {
        let ray = RayData {
            ray: get_ray(to_uv(x, y, size), plane_distance, camera),
            steps_left: steps,
            refraction_stack: rpds::Stack::new().push(1.0),
        };

        cast_ray(ray, &config)
    };

    #[cfg(feature = "wasm")]
    return Fb::from_func(size.0, size.1, func);

    #[cfg(not(feature = "wasm"))]
    return Fb::from_par_func(size.0, size.1, func);
}

fn to_uv(x: u16, y: u16, size: (u16, u16)) -> (f32, f32) {
    (
        (x as f32 / size.0 as f32 * 2.0 - 1.0) * (size.0 as f32 / size.1 as f32),
        -(y as f32 / size.1 as f32 * 2.0 - 1.0),
    )
}

fn get_ray(uv: (f32, f32), plane_distance: f32, camera: Isometry3<f32>) -> Ray<f32> {
    let direction = Vector3::new(uv.0, uv.1, plane_distance);
    Ray::new(Point3::origin(), direction).transform_by(&camera)
}

fn cast_ray(ray: RayData, config: &RtConfig) -> Color {
    if ray.steps_left == 0 {
        return Color::black();
    }

    if let Some((obj, int)) = first_interference(ray.ray, &config.world) {
        let normal = Ray::new(ray.ray.origin + ray.ray.dir * int.toi, int.normal);
        get_color(
            ray,
            config,
            GetColorArgs {
                normal,
                mat: &obj.data().mat,
            },
        )
    } else {
        config.ambient
    }
}

fn first_interference(
    ray: Ray<f32>,
    world: &CollisionWorld,
) -> Option<(&CollisionObject, RayIntersection<f32>)> {
    world
        .interferences_with_ray(&ray, &CollisionGroups::new())
        .min_by(|(_, _, isect1), (_, _, isect2)| isect1.toi.partial_cmp(&isect2.toi).unwrap_or(std::cmp::Ordering::Less))
        .map(|(_, obj, isect)| (obj, isect))
}

#[derive(Clone, Copy)]
struct GetColorArgs<'a> {
    mat: &'a Material,
    normal: Ray<f32>,
}

fn get_color(ray: RayData, config: &RtConfig, args: GetColorArgs) -> Color {
    let GetColorArgs {
        mat: Material { phong, reflect },
        normal,
    } = args;

    let mut color = phong.ambient * config.ambient;

    let origin_with_margin = normal.origin + normal.dir * 0.00001;

    let viewer = -ray.ray.dir;
    let viewer_reflection = {
        let rotation_to_normal = UnitQuaternion::rotation_between(&viewer, &normal.dir).unwrap();
        rotation_to_normal * rotation_to_normal * viewer
    };
    let reflection_color = cast_ray(
        ray.push(Ray::new(origin_with_margin, viewer_reflection)),
        config,
    );

    for light in &config.lights {
        let light_pos = Point3::from(light.pos.translation.vector);
        let distance = na::distance(&normal.origin, &light_pos);

        let intersection = first_interference(
            Ray::new(origin_with_margin, light_pos - origin_with_margin),
            &config.world,
        );

        let light_is_visible = if let Some((_, intersection)) = intersection {
            intersection.toi > distance
        } else {
            true
        };

        if light_is_visible {
            let light_brightness = match light.kind {
                LightSourceKind::Point => light.brightness,
            };
            let light_dir = light_pos - normal.origin;
            let light_reflection = {
                let light_direction = light_pos - normal.origin;
                let rotation_to_normal =
                    UnitQuaternion::rotation_between(&light_direction, &normal.dir).unwrap();
                rotation_to_normal * rotation_to_normal * light_direction
            };

            let diffuse =
                phong.diffuse * light_brightness * normal.dir.angle(&light_dir).cos().max(0.0);

            let specular = (phong.specular * light_brightness).combine(&phong.shininess, |spec, shine| {
                spec * viewer.angle(&light_reflection).cos().max(0.0).powf(shine)
            });

            let phong_color = (diffuse + specular) * na::distance_squared(&normal.origin, &light_pos).recip();

            color = color + phong_color * phong.part;
        }
    }

    color + reflection_color * reflect.part
}

#[cfg(test)]
mod test {
    use super::to_uv;

    #[test]
    fn center_left() {
        assert_eq!(to_uv(0, 10, (30, 20)), (-1.5, 0.0));
    }

    #[test]
    fn top_left() {
        assert_eq!(to_uv(0, 0, (30, 20)), (-1.5, 1.0));
    }

    #[test]
    fn top_left_over_two() {
        assert_eq!(to_uv(10, 5, (40, 20)), (-1.0, 0.5))
    }
}
