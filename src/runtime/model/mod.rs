// Copyright 2024 the Velato Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::ops::Range;
use serde::{Deserialize, Serialize};
use kurbo::{self, Affine, PathEl, Point, Shape as _};
use peniko::{self, BlendMode, Color};

mod spline;
mod value;

pub mod animated;
pub mod fixed;

pub use value::{Animated, Easing, EasingHandle, Time, Tween, Value, ValueRef};

pub(crate) use spline::SplineToPath;

use crate::PathEl32;

macro_rules! simple_value {
    ($name:ident) => {
        #[allow(clippy::large_enum_variant)]
        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub enum $name {
            Fixed(fixed::$name),
            Animated(animated::$name),
        }

        impl $name {
            pub fn is_fixed(&self) -> bool {
                matches!(self, Self::Fixed(_))
            }
            pub fn evaluate(&self, frame: f64) -> ValueRef<fixed::$name> {
                match self {
                    Self::Fixed(value) => ValueRef::Borrowed(value),
                    Self::Animated(value) => ValueRef::Owned(value.evaluate(frame)),
                }
            }
        }
    };
}

simple_value!(Transform);
simple_value!(Stroke);
simple_value!(Repeater);
simple_value!(ColorStops);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Brush {
    Fixed(fixed::Brush),
    Animated(animated::Brush),
}

impl Brush {
    pub fn is_fixed(&self) -> bool {
        matches!(self, Self::Fixed(_))
    }

    pub fn evaluate(&self, alpha: f64, frame: f64) -> ValueRef<fixed::Brush> {
        match self {
            Self::Fixed(value) => {
                if alpha == 1.0 {
                    ValueRef::Borrowed(value)
                } else {
                    ValueRef::Owned(fixed::brush_with_alpha(value, alpha))
                }
            }
            Self::Animated(value) => ValueRef::Owned(value.evaluate(alpha, frame)),
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::Fixed(Affine::IDENTITY)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Geometry {
    Fixed(Vec<PathEl32>),
    Rect(animated::Rect),
    Ellipse(animated::Ellipse),
    Spline(animated::Spline),
}

impl Geometry {
    pub fn evaluate(&self, frame: f64, path: &mut Vec<PathEl>) {
        match self {
            Self::Fixed(value) => {
                path.extend(value.iter().map(|el| el.to_path_el()));
            }
            Self::Rect(value) => {
                path.extend(value.evaluate(frame).path_elements(0.1));
            }
            Self::Ellipse(value) => {
                path.extend(value.evaluate(frame).path_elements(0.1));
            }
            Self::Spline(value) => {
                value.evaluate(frame, path);
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Draw {
    /// Parameters for a stroked draw operation.
    pub stroke: Option<Stroke>,
    /// Brush for the draw operation.
    pub brush: Brush,
    /// Opacity of the draw operation.
    pub opacity: Value<f32>,
}

/// Elements of a shape layer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Shape {
    /// Group of shapes with an optional transform.
    Group(Vec<Shape>, Option<GroupTransform>),
    /// Geometry element.
    Geometry(Geometry),
    /// Fill or stroke element.
    Draw(Draw),
    /// Repeater element.
    Repeater(Repeater),
}

/// Transform and opacity for a shape group.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupTransform {
    pub transform: Transform,
    pub opacity: Value<f32>,
}

/// Layer in an animation.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Layer {
    /// Name of the layer.
    pub name: String,
    /// Index of the transform parent layer.
    pub parent: Option<usize>,
    /// Transform for the entire layer.
    pub transform: Transform,
    /// Opacity for the entire layer.
    pub opacity: Value<f32>,
    /// Width of the layer.
    pub width: f64,
    /// Height of the layer.
    pub height: f64,
    /// Blend mode for the layer.
    pub blend_mode: Option<peniko::BlendMode>,
    /// Range of frames in which the layer is active.
    pub frames: Range<f64>,
    /// Frame time stretch factor.
    pub stretch: f64,
    /// Starting frame for the layer (only applied to instances).
    pub start_frame: f64,
    /// List of masks applied to the content.
    pub masks: Vec<Mask>,
    /// True if the layer is used as a mask.
    pub is_mask: bool,
    /// Mask blend mode and layer.
    pub mask_layer: Option<(BlendMode, usize)>,
    /// Content of the layer.
    pub content: Content,
}

/// Matte layer mode.
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub enum Matte {
    #[default]
    Normal,
    // TODO: Use these
    // Alpha,
    // InvertAlpha,
    // Luma,
    // InvertLuma,
}

/// Mask for a layer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mask {
    /// Blend mode for the mask.
    pub mode: peniko::BlendMode,
    /// Geometry that defines the shape of the mask.
    pub geometry: Geometry,
    /// Opacity of the mask.
    pub opacity: Value<f32>,
}

/// Content of a layer.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub enum Content {
    /// Empty layer.
    #[default]
    None,
    /// Asset instance with the specified name and time remapping.
    Instance {
        name: String,
        time_remap: Option<Value<f32>>,
    },
    /// Collection of shapes.
    Shape(Vec<Shape>),
}
