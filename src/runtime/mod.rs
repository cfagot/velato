// Copyright 2024 the Velato Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

mod render;

use crate::import;
use crate::schema::Animation;
use crate::Error;
use std::collections::HashMap;
use std::ops::Range;

pub mod model;

pub use render::Renderer;
use serde::{Deserialize, Serialize};

/// Model of a Lottie file.
#[derive(Clone, Default, Debug)]
pub struct Composition {
    /// Frames in which the animation is active.
    pub frames: Range<f64>,
    /// Frames per second.
    pub frame_rate: f64,
    /// Width of the animation.
    pub width: usize,
    /// Height of the animation.
    pub height: usize,
    /// Precomposed layers that may be instanced.
    pub assets: HashMap<String, Vec<model::Layer>>,
    /// Collection of layers.
    pub layers: Vec<model::Layer>,
}

impl Composition {
    /// Creates a new runtime composition from a buffer of Lottie file contents.
    pub fn from_slice(source: impl AsRef<[u8]>) -> Result<Composition, Error> {
        let source = Animation::from_slice(source.as_ref())?;
        let composition = import::conv_animation(source);
        Ok(composition)
    }

    /// Creates a new runtime composition from a json object of Lottie file contents.
    pub fn from_json(v: serde_json::Value) -> Result<Composition, Error> {
        let source = Animation::from_json(v)?;
        let composition = import::conv_animation(source);
        Ok(composition)
    }
}

impl std::str::FromStr for Composition {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let source = Animation::from_str(s)?;
        let composition = import::conv_animation(source);
        Ok(composition)
    }
}

// Mirror type of Composition with serde annotations.
// Can't add serde to Composition because it would cause confusion
// with the built in serialization methods (from_str, from json, from_slice)
// which are not compatible (they assume lottie file, this is serialization of
// internal representation and is not stable across builds).
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct CompositionSerde {
    /// Frames in which the animation is active.
    pub frames: Range<f64>,
    /// Frames per second.
    pub frame_rate: f64,
    /// Width of the animation.
    pub width: usize,
    /// Height of the animation.
    pub height: usize,
    /// Precomposed layers that may be instanced.
    pub assets: HashMap<String, Vec<model::Layer>>,
    /// Collection of layers.
    pub layers: Vec<model::Layer>,
}

impl CompositionSerde {
    pub fn to_serde(composition: Composition) -> CompositionSerde {
        CompositionSerde {
            frames: composition.frames,
            frame_rate: composition.frame_rate,
            width: composition.width,
            height: composition.height,
            assets: composition.assets,
            layers: composition.layers,
        }
    }

    pub fn from_serde(composition: CompositionSerde) -> Composition {
        Composition {
            frames: composition.frames,
            frame_rate: composition.frame_rate,
            width: composition.width,
            height: composition.height,
            assets: composition.assets,
            layers: composition.layers,
        }
    }
}