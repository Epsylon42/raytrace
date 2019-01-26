use serde::{Serialize, Deserialize};
use crate::fb::Color;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Material {
    #[serde(default)]
    pub phong: Phong,
    #[serde(default)]
    pub reflect: Reflect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phong {
    #[serde(default = "default_phong_part")]
    pub part: f32,
    #[serde(default = "Color::white")]
    pub ambient: Color,
    #[serde(default = "Color::white")]
    pub diffuse: Color,
    #[serde(default = "Color::black")]
    pub specular: Color,
    #[serde(default = "Color::black")]
    pub shininess: Color,
}

impl Default for Phong {
    fn default() -> Self {
        Phong {
            part: 1.0,
            ambient: Color::white(),
            diffuse: Color::white(),
            specular: Color::black(),
            shininess: Color::black(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Reflect {
    #[serde(default)]
    pub part: f32,
}

fn default_phong_part() -> f32 {
    1.0
}
