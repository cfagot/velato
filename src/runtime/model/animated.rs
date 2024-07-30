// Copyright 2024 the Velato Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

/*!
Representations of animated values.
*/

use crate::{PointF32, SizeF32, VecF32};

use super::*;

use kurbo::PathEl;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Position {
    Value(Value<PointF32>),
    SplitValues((Value<f32>, Value<f32>)),
}

/// Animated affine transformation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transform {
    /// Anchor point.
    pub anchor: Value<PointF32>,
    /// Translation.
    pub position: Position,
    /// Rotation angle.
    pub rotation: Value<f32>,
    /// Scale factor.
    pub scale: Value<VecF32>,
    /// Skew factor.
    pub skew: Value<f32>,
    /// Skew angle.
    pub skew_angle: Value<f32>,
}

impl Transform {
    /// Returns true if the transform is fixed.
    pub fn is_fixed(&self) -> bool {
        self.anchor.is_fixed()
            && match &self.position {
                Position::Value(value) => value.is_fixed(),
                Position::SplitValues((x_value, y_value)) => {
                    x_value.is_fixed() && y_value.is_fixed()
                }
            }
            && self.rotation.is_fixed()
            && self.scale.is_fixed()
            && self.skew.is_fixed()
            && self.skew_angle.is_fixed()
    }

    /// Evaluates the transform at the specified frame.
    pub fn evaluate(&self, frame: f64) -> Affine {
        let anchor = self.anchor.evaluate(frame);
        let position = match &self.position {
            Position::Value(value) => value.evaluate(frame),
            Position::SplitValues((x_value, y_value)) => PointF32::new(
                x_value.evaluate(frame),
                y_value.evaluate(frame),
            ),
        };
        let rotation = self.rotation.evaluate(frame);
        let scale = self.scale.evaluate(frame);
        let skew = self.skew.evaluate(frame);
        let skew_angle = self.skew_angle.evaluate(frame);
        let skew_matrix = if skew != 0.0 {
            const SKEW_LIMIT: f64 = 85.0;
            let skew = -skew.clamp(-SKEW_LIMIT as f32, SKEW_LIMIT as f32);
            let skew = skew.to_radians();
            let angle = skew_angle.to_radians();
            Affine::rotate(-angle as f64) * Affine::skew(skew.tan() as f64, 0.0) * Affine::rotate(angle as f64)
        } else {
            Affine::IDENTITY
        };
        Affine::translate(kurbo::Vec2::new(position.x as f64, position.y as f64))
            * Affine::rotate(rotation.to_radians() as f64)
            * skew_matrix
            * Affine::scale_non_uniform(scale.x as f64 / 100.0, scale.y as f64/ 100.0)
            * Affine::translate((-anchor.x as f64, -anchor.y as f64))
    }

    /// Converts the animated value to its model representation.
    pub fn into_model(self) -> super::Transform {
        if self.is_fixed() {
            super::Transform::Fixed(self.evaluate(0.0))
        } else {
            super::Transform::Animated(self)
        }
    }
}

/// Animated ellipse.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ellipse {
    /// True if the ellipse should be drawn in CCW order.
    pub is_ccw: bool,
    /// Position of the ellipse.
    pub position: Value<PointF32>,
    /// Size of the ellipse.
    pub size: Value<SizeF32>,
}

impl Ellipse {
    pub fn is_fixed(&self) -> bool {
        self.position.is_fixed() && self.size.is_fixed()
    }

    pub fn evaluate(&self, frame: f64) -> kurbo::Ellipse {
        let position = self.position.evaluate(frame).to_point();
        let size = self.size.evaluate(frame).to_size();
        let radii = (size.width * 0.5, size.height * 0.5);
        kurbo::Ellipse::new(position, radii, 0.0)
    }
}

/// Animated rounded rectangle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rect {
    /// True if the rect should be drawn in CCW order.
    pub is_ccw: bool,
    /// Position of the rectangle.
    pub position: Value<PointF32>,
    /// Size of the rectangle.
    pub size: Value<SizeF32>,
    /// Radius of the rectangle corners.
    pub corner_radius: Value<f32>,
}

impl Rect {
    /// Returns true if the rectangle is fixed.
    pub fn is_fixed(&self) -> bool {
        self.position.is_fixed() && self.size.is_fixed() && self.corner_radius.is_fixed()
    }

    /// Evaluates the rectangle at the specified frame.
    pub fn evaluate(&self, frame: f64) -> kurbo::RoundedRect {
        let position = self.position.evaluate(frame);
        let size = self.size.evaluate(frame);
        let position = Point::new(
            (position.x - size.width * 0.5) as f64,
            (position.y - size.height * 0.5) as f64,
        );
        let radius = self.corner_radius.evaluate(frame);
        kurbo::RoundedRect::new(
            position.x,
            position.y,
            position.x + size.width as f64,
            position.y + size.height as f64,
            radius as f64,
        )
    }
}

/// Animated star or polygon.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Star {
    pub is_polygon: bool,
    pub direction: f64,
    pub position: Value<PointF32>,
    pub inner_radius: Value<f32>,
    pub inner_roundness: Value<f32>,
    pub outer_radius: Value<f32>,
    pub outer_roundness: Value<f32>,
    pub rotation: Value<f32>,
    pub points: Value<f32>,
}

// TODO: Use this.
//impl Star {
//    pub fn is_fixed(&self) -> bool {
//        self.position.is_fixed()
//            && self.inner_radius.is_fixed()
//            && self.inner_roundness.is_fixed()
//            && self.outer_radius.is_fixed()
//            && self.outer_roundness.is_fixed()
//            && self.rotation.is_fixed()
//            && self.points.is_fixed()
//    }
//}

/// Animated cubic spline.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Spline {
    /// True if the spline is closed.
    pub is_closed: bool,
    /// Collection of times.
    pub times: Vec<Time>,
    /// Collection of splines.
    pub values: Vec<Vec<PointF32>>,
}

impl Spline {
    /// Evaluates the spline at the given frame and emits the elements
    /// to the specified path.
    pub fn evaluate(&self, frame: f64, path: &mut Vec<PathEl>) -> bool {
        let Some(([ix0, ix1], t, _easing, _hold)) = Time::frames_and_weight(&self.times, frame)
        else {
            // TODO: evaluate whether hold frame is needed here
            return false;
        };
        let (Some(from), Some(to)) = (self.values.get(ix0), self.values.get(ix1)) else {
            return false;
        };
        (from.as_slice(), to.as_slice(), t).to_path(self.is_closed, path);
        true
    }
}

/// Animated repeater effect.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Repeater {
    /// Number of times elements should be repeated.
    pub copies: Value<f32>,
    /// Offset applied to each element.
    pub offset: Value<f32>,
    /// Anchor point.
    pub anchor_point: Value<PointF32>,
    /// Translation.
    pub position: Value<PointF32>,
    /// Rotation in degrees.
    pub rotation: Value<f32>,
    /// Scale.
    pub scale: Value<VecF32>,
    /// Opacity of the first element.
    pub start_opacity: Value<f32>,
    /// Opacity of the last element.
    pub end_opacity: Value<f32>,
}

impl Repeater {
    /// Returns true if the repeater contains no animated properties.
    pub fn is_fixed(&self) -> bool {
        self.copies.is_fixed()
            && self.offset.is_fixed()
            && self.anchor_point.is_fixed()
            && self.position.is_fixed()
            && self.rotation.is_fixed()
            && self.scale.is_fixed()
            && self.start_opacity.is_fixed()
            && self.end_opacity.is_fixed()
    }

    /// Evaluates the repeater at the specified frame.
    pub fn evaluate(&self, frame: f64) -> fixed::Repeater {
        let copies = self.copies.evaluate(frame).round() as usize;
        let offset = self.offset.evaluate(frame) as f64;
        let anchor_point = self.anchor_point.evaluate(frame).to_point();
        let position = self.position.evaluate(frame).to_point();
        let rotation = self.rotation.evaluate(frame) as f64;
        let scale = self.scale.evaluate(frame).to_vec2();
        let start_opacity = self.start_opacity.evaluate(frame) as f64;
        let end_opacity = self.end_opacity.evaluate(frame) as f64;
        fixed::Repeater {
            copies,
            offset,
            anchor_point,
            position,
            rotation,
            scale,
            start_opacity,
            end_opacity,
        }
    }

    /// Converts the animated value to its model representation.
    pub fn into_model(self) -> super::Repeater {
        if self.is_fixed() {
            super::Repeater::Fixed(self.evaluate(0.0))
        } else {
            super::Repeater::Animated(self)
        }
    }
}

/// Animated stroke properties.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stroke {
    /// Width of the stroke.
    pub width: Value<f32>,
    /// Join style.
    pub join: kurbo::Join,
    /// Limit for miter joins.
    pub miter_limit: Option<f64>,
    /// Cap style.
    pub cap: kurbo::Cap,
}

impl Stroke {
    /// Returns true if the stroke is fixed.
    pub fn is_fixed(&self) -> bool {
        self.width.is_fixed()
    }

    /// Evaluates the stroke at the specified frame.
    pub fn evaluate(&self, frame: f64) -> kurbo::Stroke {
        let width = self.width.evaluate(frame);
        let mut stroke = kurbo::Stroke::new(width as f64)
            .with_caps(self.cap)
            .with_join(self.join);
        if let Some(miter_limit) = self.miter_limit {
            stroke.miter_limit = miter_limit;
        }
        stroke
    }

    /// Converts the animated value to its model representation.
    pub fn into_model(self) -> super::Stroke {
        if self.is_fixed() {
            super::Stroke::Fixed(self.evaluate(0.0))
        } else {
            super::Stroke::Animated(self)
        }
    }
}

/// Animated linear or radial gradient.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gradient {
    /// True if the gradient is radial.
    pub is_radial: bool,
    /// Starting point.
    pub start_point: Value<PointF32>,
    /// Ending point.
    pub end_point: Value<PointF32>,
    /// Stop offsets and color values.
    pub stops: super::ColorStops,
}

impl Gradient {
    /// Returns true if the value contains no animated properties.
    pub fn is_fixed(&self) -> bool {
        self.start_point.is_fixed() && self.end_point.is_fixed() && self.stops.is_fixed()
    }

    /// Evaluates the animated value at the given frame.
    pub fn evaluate(&self, frame: f64) -> peniko::Brush {
        let start = self.start_point.evaluate(frame);
        let end = self.end_point.evaluate(frame);
        let stops = self.stops.evaluate(frame).into_owned();
        if self.is_radial {
            let radius = (end.to_vec2() - start.to_vec2()).hypot();
            let mut grad = peniko::Gradient::new_radial(start.to_point(), radius as f32);
            grad.stops = stops;
            grad.into()
        } else {
            let mut grad = peniko::Gradient::new_linear(start.to_point(), end.to_point());
            grad.stops = stops;
            grad.into()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorStops {
    pub frames: Vec<Time>,
    pub values: Vec<Vec<f32>>,
    pub count: usize,
}

impl ColorStops {
    pub fn evaluate(&self, frame: f64) -> fixed::ColorStops {
        self.evaluate_inner(frame).unwrap_or_default()
    }

    fn evaluate_inner(&self, frame: f64) -> Option<fixed::ColorStops> {
        let ([ix0, ix1], t, easing, hold) = Time::frames_and_weight(&self.frames, frame)?;

        let v0 = self.values.get(ix0)?;
        let v1 = self.values.get(ix1)?;

        let mut stops: fixed::ColorStops = Default::default();
        for i in 0..self.count {
            let j = i * 5;
            let offset = v0.get(j)?.tween(v1.get(j)?, t, &easing);
            let t = if hold { 0.0 } else { t };
            let r = v0.get(j + 1)?.tween(v1.get(j + 1)?, t, &easing) as f64;
            let g = v0.get(j + 2)?.tween(v1.get(j + 2)?, t, &easing) as f64;
            let b = v0.get(j + 3)?.tween(v1.get(j + 3)?, t, &easing) as f64;
            let a = v0.get(j + 4)?.tween(v1.get(j + 4)?, t, &easing) as f64;
            let stop = peniko::ColorStop::from((offset as f32, peniko::Color::rgba(r, g, b, a)));
            stops.push(stop);
        }
        Some(stops)
    }
}

/// Animated brush.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Brush {
    /// Solid color.
    Solid(Value<Color>),
    /// Gradient color.
    Gradient(Gradient),
}

impl Brush {
    /// Returns true if the value contains no animated properties.
    pub fn is_fixed(&self) -> bool {
        match self {
            Self::Solid(value) => value.is_fixed(),
            Self::Gradient(value) => value.is_fixed(),
        }
    }

    /// Evaluates the animation at the specified time.
    pub fn evaluate(&self, alpha: f64, frame: f64) -> fixed::Brush {
        match self {
            Self::Solid(value) => value.evaluate(frame).with_alpha_factor(alpha as f32).into(),
            Self::Gradient(value) => value.evaluate(frame),
        }
    }

    /// Converts the animated value to its model representation.
    pub fn into_model(self) -> super::Brush {
        if self.is_fixed() {
            super::Brush::Fixed(self.evaluate(1.0, 0.0))
        } else {
            super::Brush::Animated(self)
        }
    }
}
