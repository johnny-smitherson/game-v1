use std::marker::PhantomData;

use bevy::prelude::*;

use crate::DebugLines;

pub use self::circle::Circle;
pub use self::cuboid::Cuboid;
pub use self::line::Line;
pub use self::rect::Rect;
pub use self::sphere::Sphere;

mod circle;
mod cuboid;
mod line;
mod rect;
mod sphere;

/// Bevy resource providing facilities to draw shapes.
///
/// # Usage
/// ```
/// use bevy::prelude::*;
/// use bevy_prototype_debug_lines::*;
///
/// // Draws a red cuboid (box) rotating around X.
/// fn some_system(time: Res<Time>, mut shapes: ResMut<DebugShapes>) {
///     let seconds = time.elapsed_seconds();
///
///     shapes
///         .cuboid()
///         .rotation(Quat::from_axis_angle(
///             Vec3::X,
///             seconds * std::f32::consts::FRAC_PI_4,
///         ))
///         .color(Color::RED);
/// }
/// ```
#[derive(Resource, Default)]
pub struct DebugShapes {
    pub shapes: Vec<Shape>,
}

impl DebugShapes {
    /// Add a generic shape to be drawn and return a handle to it.
    pub fn add<S>(&mut self, shape: S) -> ShapeHandle<'_, S>
    where
        S: Into<Shape>,
    {
        let index = self.shapes.len();
        self.shapes.push(shape.into());
        ShapeHandle::new(self, index)
    }

    /// Adds a [`Circle`] shape.
    ///
    /// See [`ShapeHandle`] impl on [`Circle`] for more shape properties.
    ///
    /// Short for [`DebugShapes::add`].
    pub fn circle(&mut self) -> ShapeHandle<'_, Circle> {
        self.add(Circle::new())
    }

    /// Adds a [`Cuboid`] shape.
    ///
    /// See [`ShapeHandle`] impl on [`Cuboid`] for more shape properties.
    ///
    /// Short for [`DebugShapes::add`].
    pub fn cuboid(&mut self) -> ShapeHandle<'_, Cuboid> {
        self.add(Cuboid::new())
    }

    /// Adds a [`Line`] shape.
    ///
    /// See [`ShapeHandle`] impl on [`Line`] for more shape properties.
    ///
    /// Short for [`DebugShapes::add`].
    pub fn line(&mut self) -> ShapeHandle<'_, Line> {
        self.add(Line::new())
    }

    /// Adds a [`Sphere`] shape.
    ///
    /// See [`ShapeHandle`] impl on [`Sphere`] for more shape properties.
    ///
    /// Short for [`DebugShapes::add`].s
    pub fn sphere(&mut self) -> ShapeHandle<'_, Sphere> {
        self.add(Sphere::new())
    }

    /// Adds a [`Rect`] shape.
    ///
    /// See [`ShapeHandle`] impl on [`Rect`] for more shape properties.
    ///
    /// Short for [`DebugShapes::add`].s
    pub fn rect(&mut self) -> ShapeHandle<'_, Rect> {
        self.add(Rect::new())
    }
}

/// Implemented on shapes to add lines to [`DebugLines`].
pub(crate) trait AddLines {
    /// Add required lines to [`DebugLines`] for drawing shape.
    fn add_lines(&self, lines: &mut DebugLines);
}

/// Wrapper around all shape types to allow matching to specific shapes.
pub enum Shape {
    Circle(Circle),
    Cuboid(Cuboid),
    Line(Line),
    Rect(Rect),
    Sphere(Sphere),
}

impl AddLines for Shape {
    fn add_lines(&self, lines: &mut DebugLines) {
        match self {
            Shape::Circle(s) => s.add_lines(lines),
            Shape::Cuboid(s) => s.add_lines(lines),
            Shape::Line(s) => s.add_lines(lines),
            Shape::Rect(s) => s.add_lines(lines),
            Shape::Sphere(s) => s.add_lines(lines),
        }
    }
}

/// Used to modify shapes after they've been added to [`DebugShapes`].
pub struct ShapeHandle<'a, S> {
    pub(crate) shapes: &'a mut DebugShapes,
    pub(crate) index: usize,
    _ty: PhantomData<S>,
}

impl<'a, S> ShapeHandle<'a, S> {
    pub(crate) fn new(shapes: &'a mut DebugShapes, index: usize) -> Self {
        Self {
            shapes,
            index,
            _ty: PhantomData,
        }
    }
}
