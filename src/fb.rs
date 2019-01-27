use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b }
    }

    pub fn black() -> Self {
        Color::new(0.0, 0.0, 0.0)
    }
    pub fn white() -> Self {
        Color::new(1.0, 1.0, 1.0)
    }

    pub fn to_pixel(&self) -> [u8; 3] {
        [
            (self.r.max(0.0).min(1.0) * 255.0) as u8,
            (self.g.max(0.0).min(1.0) * 255.0) as u8,
            (self.b.max(0.0).min(1.0) * 255.0) as u8,
        ]
    }

    pub fn map(&self, f: impl Fn(f32) -> f32) -> Self {
        Color {
            r: f(self.r),
            g: f(self.g),
            b: f(self.b),
        }
    }

    pub fn combine(&self, other: &Self, f: impl Fn(f32, f32) -> f32) -> Self {
        Color {
            r: f(self.r, other.r),
            g: f(self.g, other.g),
            b: f(self.b, other.b),
        }
    }
}

impl std::ops::Mul<f32> for Color {
    type Output = Self;

    fn mul(self, other: f32) -> Self::Output {
        Color {
            r: self.r * other,
            g: self.g * other,
            b: self.b * other,
        }
    }
}

impl std::ops::Mul<Self> for Color {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Color {
            r: self.r * other.r,
            g: self.g * other.g,
            b: self.b * other.b,
        }
    }
}

impl std::ops::Add<Self> for Color {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Color {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }
}

pub struct Fb {
    width: u16,
    height: u16,
    data: Vec<Color>,
}

impl Fb {
    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn new_empty(width: u16, height: u16) -> Self {
        Fb {
            width,
            height,
            data: vec![Color::new(0.0, 0.0, 0.0); (width * height) as usize],
        }
    }

    pub fn from_func(width: u16, height: u16, func: impl Fn(u16, u16) -> Color) -> Self {
        let func = &func;

        Fb {
            width,
            height,
            data: (0..height)
                .flat_map(|y| (0..width).map(move |x| func(x, y)))
                .collect(),
        }
    }

    pub fn from_par_func(width: u16, height: u16, func: impl Fn(u16, u16) -> Color + Sync) -> Self {
        let func = &func;

        Fb {
            width,
            height,
            data: (0..height)
                .into_par_iter()
                .flat_map(|y| (0..width).into_par_iter().map(move |x| func(x, y)))
                .collect(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.data
            .iter()
            .map(Color::to_pixel)
            .flat_map(|[r, g, b]| {
                std::iter::empty()
                    .chain(std::iter::once(r))
                    .chain(std::iter::once(g))
                    .chain(std::iter::once(b))
            })
            .collect()
    }

    pub fn get(&self, x: u16, y: u16) -> Color {
        self.data[pack(x, y, self.width)]
    }
}

fn pack(x: u16, y: u16, width: u16) -> usize {
    (y as usize * width as usize) + x as usize
}

fn unpack(i: usize, width: u16) -> (u16, u16) {
    ((i % width as usize) as u16, (i / width as usize) as u16)
}
