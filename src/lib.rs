// Copyright 2024 the Velato Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Render a Lottie animation to a Vello [`Scene`](crate::vello::Scene).
//!
//! This currently lacks support for a [number of important](crate#unsupported-features) SVG features.
//!
//! This is also intended to be the preferred integration between Vello and [usvg], so [consider
//! contributing](https://github.com/linebender/vello_svg) if you need a feature which is missing.
//!
//! This crate also re-exports [`vello`], so you can easily use the specific version that is compatible with Velato.
//!
//! ## Usage
//!
//! ```no_run
//! # use std::str::FromStr;
//! use velato::vello;
//!
//! // Parse your lottie file
//! let lottie = include_str!("../examples/assets/google_fonts/Tiger.json");
//! let composition = velato::Composition::from_str(lottie).expect("valid file");
//!
//! // Render to a scene
//! let mut new_scene = vello::Scene::new();
//!
//! // Render to a scene!
//! let mut renderer = velato::Renderer::new();
//! let frame = 0.0; // Arbitrary number chosen. Ensure it's a valid frame!
//! let transform = vello::kurbo::Affine::IDENTITY;
//! let alpha = 1.0;
//! let scene = renderer.render(&composition, frame, transform, alpha);
//! ```
//!
//! # Unsupported features
//!
//! Missing features include:
//! - Non-linear easings
//! - Position keyframe (`ti`, `to`) easing
//! - Time remapping (`tm`)
//! - Text
//! - Image embedding
//! - Advanced shapes (stroke dash, zig-zag, etc.)
//! - Advanced effects (motion blur, drop shadows, etc.)
//! - Correct color stop handling
//! - Split rotations
//! - Split positions

pub(crate) mod import;
pub(crate) mod runtime;
pub(crate) mod schema;

mod error;
pub use error::Error;

// Re-export vello
pub use vello;

pub use runtime::{model, Composition, CompositionSerde, Renderer};

pub type VecF32 = PointF32;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct PointF32 {
    x: f32,
    y: f32,
}

impl PointF32 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn to_point(&self) -> kurbo::Point {
        kurbo::Point::new(self.x as f64, self.y as f64)
    }

    pub fn to_vec2(&self) -> kurbo::Vec2 {
        kurbo::Vec2::new(self.x as f64, self.y as f64)
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SizeF32 {
    width: f32,
    height: f32,
}

impl SizeF32 {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn to_size(&self) -> kurbo::Size {
        kurbo::Size::new(self.width as f64, self.height as f64)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum PathEl32 {
    /// Move directly to the point without drawing anything, starting a new
    /// subpath.
    MoveTo(PointF32),
    /// Draw a line from the current location to the point.
    LineTo(PointF32),
    /// Draw a quadratic bezier using the current location and the two points.
    QuadTo(PointF32, PointF32),
    /// Draw a cubic bezier using the current location and the three points.
    CurveTo(PointF32, PointF32, PointF32),
    /// Close off the path.
    ClosePath,
}

impl PathEl32 {
    pub fn to_path_el(&self) -> kurbo::PathEl {
        match self {
            Self::MoveTo(p) => kurbo::PathEl::MoveTo(p.to_point()),
            Self::LineTo(p) => kurbo::PathEl::LineTo(p.to_point()),
            Self::QuadTo(p1, p2) => kurbo::PathEl::QuadTo(p1.to_point(), p2.to_point()),
            Self::CurveTo(p1, p2, p3) => kurbo::PathEl::CurveTo(
                p1.to_point(),
                p2.to_point(),
                p3.to_point(),
            ),
            Self::ClosePath => kurbo::PathEl::ClosePath,
        }
    }

    pub fn from_f64(path_el: &kurbo::PathEl) -> Self {
        match path_el {
            kurbo::PathEl::MoveTo(p) => Self::MoveTo(PointF32::new(p.x as f32, p.y as f32)),
            kurbo::PathEl::LineTo(p) => Self::LineTo(PointF32::new(p.x as f32, p.y as f32)),
            kurbo::PathEl::QuadTo(p1, p2) => Self::QuadTo(
                PointF32::new(p1.x as f32, p1.y as f32),
                PointF32::new(p2.x as f32, p2.y as f32),
            ),
            kurbo::PathEl::CurveTo(p1, p2, p3) => Self::CurveTo(
                PointF32::new(p1.x as f32, p1.y as f32),
                PointF32::new(p2.x as f32, p2.y as f32),
                PointF32::new(p3.x as f32, p3.y as f32),
            ),
            kurbo::PathEl::ClosePath => Self::ClosePath,
        }
    }
}
