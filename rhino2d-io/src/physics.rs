use serde::{Deserialize, Serialize};

/// Model-wide physics properties.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Physics {
    pixels_per_meter: f32,
    gravity: f32,
}

impl Physics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pixels_per_meter(&self) -> f32 {
        self.pixels_per_meter
    }

    pub fn set_pixels_per_meter(&mut self, pixels_per_meter: f32) {
        self.pixels_per_meter = pixels_per_meter;
    }

    pub fn gravity(&self) -> f32 {
        self.gravity
    }

    pub fn set_gravity(&mut self, gravity: f32) {
        self.gravity = gravity;
    }
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            pixels_per_meter: 1000.0,
            gravity: 9.8,
        }
    }
}
