// use std::collections::{vec_deque, VecDeque};
use bevy::prelude::*;

use super::terrain::PLANET_RADIUS;
use crate::triangle::Triangle;

pub trait Piramidesc {
    fn base_tris(&mut self) -> Vec<Triangle>;
}

/// AM PIRAMIDĂ
#[derive(Debug, Clone)]
pub struct Piramidă<const N: usize> {
    pub children: [Triangle; N],
}

impl<const N: usize> Piramidesc for Piramidă<N> {
    fn base_tris(&mut self) -> Vec<Triangle> {
        let mut vec = Vec::<Triangle>::new();
        for child in self.children.iter_mut() {
            vec.extend(child.base_tris());
        }
        vec
    }
}

impl Piramidă<1> {
    // Flat Earth Confirmed
    pub fn new() -> Self {
        // https://www.wolframalpha.com/input?i=equilateral+triangle
        let a = 1.0;
        let v1 = Vec3::new(0., 0., a / 3.0_f32.sqrt());
        let v2 = Vec3::new(-a / 2.0, 0., -a / (2. * 3.0_f32.sqrt()));
        let v3 = Vec3::new(a / 2.0, 0., -a / (2. * 3.0_f32.sqrt()));
        Self {
            children: [Triangle::new(
                [v1 * PLANET_RADIUS, v2 * PLANET_RADIUS, v3 * PLANET_RADIUS],
                0,
                1,
                "",
            )
            .reverse_points()],
        }
    }
}

impl Piramidă<4> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let v1 = Vec3::new((8.0_f32 / 9.).sqrt(), 0., -1. / 3.);
        let v2 = Vec3::new(-(2.0_f32 / 9.).sqrt(), (2.0_f32 / 3.).sqrt(), -1. / 3.);
        let v3 = Vec3::new(-(2.0_f32 / 9.).sqrt(), -(2.0_f32 / 3.).sqrt(), -1. / 3.);
        let v4 = Vec3::new(0., 0., 1.);
        Self {
            children: [
                Triangle::new([v1, v2, v3], 0, 1, "").reverse_points(),
                Triangle::new([v1, v3, v4], 0, 2, "").reverse_points(),
                Triangle::new([v2, v1, v4], 0, 3, "").reverse_points(),
                Triangle::new([v3, v2, v4], 0, 4, "").reverse_points(),
            ],
        }
    }
}

impl Piramidă<20> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let v1 = Vec3::new(0., -0.525731, 0.850651);
        let v2 = Vec3::new(0.850651, 0., 0.525731);
        let v3 = Vec3::new(0.850651, 0., -0.525731);
        let v4 = Vec3::new(-0.850651, 0., -0.525731);
        let v5 = Vec3::new(-0.850651, 0., 0.525731);
        let v6 = Vec3::new(-0.525731, 0.850651, 0.);
        let v7 = Vec3::new(0.525731, 0.850651, 0.);
        let v8 = Vec3::new(0.525731, -0.850651, 0.);
        let v9 = Vec3::new(-0.525731, -0.850651, 0.);
        let v10 = Vec3::new(0., -0.525731, -0.850651);
        let v11 = Vec3::new(0., 0.525731, -0.850651);
        let v12 = Vec3::new(0., 0.525731, 0.850651);

        Self {
            children: [
                Triangle::new([v2, v3, v7], 0, 1, ""),
                Triangle::new([v2, v8, v3], 0, 2, ""),
                Triangle::new([v4, v5, v6], 0, 3, ""),
                Triangle::new([v5, v4, v9], 0, 4, ""),
                Triangle::new([v7, v6, v12], 0, 5, ""),
                Triangle::new([v6, v7, v11], 0, 6, ""),
                Triangle::new([v10, v11, v3], 0, 7, ""),
                Triangle::new([v11, v10, v4], 0, 8, ""),
                Triangle::new([v8, v9, v10], 0, 9, ""),
                Triangle::new([v9, v8, v1], 0, 10, ""),
                Triangle::new([v12, v1, v2], 0, 11, ""),
                Triangle::new([v1, v12, v5], 0, 12, ""),
                Triangle::new([v7, v3, v11], 0, 13, ""),
                Triangle::new([v2, v7, v12], 0, 14, ""),
                Triangle::new([v4, v6, v11], 0, 15, ""),
                Triangle::new([v6, v5, v12], 0, 16, ""),
                Triangle::new([v3, v8, v10], 0, 17, ""),
                Triangle::new([v8, v2, v1], 0, 18, ""),
                Triangle::new([v4, v10, v9], 0, 19, ""),
                Triangle::new([v5, v9, v1], 0, 20, ""),
            ],
        }
    }
}
