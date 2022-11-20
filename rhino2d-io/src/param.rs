use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{Uuid, Vec2};

#[derive(Debug, Serialize, Deserialize)]
pub struct Param {
    uuid: Uuid,
    name: String,
    is_vec2: bool,
    min: Vec2,
    max: Vec2,
    defaults: Vec2,
    axis_points: Vec<Vec<f32>>,
    bindings: Vec<ParamBinding>,
}

impl Param {
    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn set_uuid(&mut self, uuid: Uuid) {
        self.uuid = uuid;
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn is_vec2(&self) -> bool {
        self.is_vec2
    }

    pub fn set_vec2(&mut self, is_vec2: bool) {
        self.is_vec2 = is_vec2;
    }

    /// Returns the minimum value of the parameter.
    ///
    /// For 1-dimensional parameters, the second value should be ignored.
    pub fn min(&self) -> Vec2 {
        self.min
    }

    pub fn set_min(&mut self, min: Vec2) {
        self.min = min;
    }

    pub fn max(&self) -> Vec2 {
        self.max
    }

    pub fn set_max(&mut self, max: Vec2) {
        self.max = max;
    }

    pub fn defaults(&self) -> Vec2 {
        self.defaults
    }

    pub fn set_defaults(&mut self, defaults: Vec2) {
        self.defaults = defaults;
    }

    /// Returns the axis points for each parameter axis.
    ///
    /// Axis points are always in range `0.0` to `1.0`. The first axis point must always be `0.0`,
    /// the last must always be `1.0`, and all axis points must be in ascending order.
    ///
    /// For 1-dimensional parameters, the second list will still be present, but only contain
    /// `[0.0]`.
    pub fn axis_points(&self) -> &[Vec<f32>] {
        &self.axis_points
    }

    pub fn set_axis_points(&mut self, axis_points: Vec<Vec<f32>>) {
        self.axis_points = axis_points;
    }

    pub fn bindings(&self) -> &[ParamBinding] {
        &self.bindings
    }

    pub fn set_bindings(&mut self, bindings: Vec<ParamBinding>) {
        self.bindings = bindings;
    }

    pub fn push_binding(&mut self, binding: ParamBinding) {
        self.bindings.push(binding);
    }

    pub fn clear_bindings(&mut self) {
        self.bindings.clear();
    }
}

/// Describes a model property affected by a [`Param`]s value.
///
/// A [`ParamBinding`] either affects a single `f32`-typed value (eg. a single axis of some
/// property), or it is of `deform` type and affects all vertices of a mesh.
///
/// Every node has the following valid properties that can be bound by a [`ParamBinding`]:
/// - `zSort`: the node's Z-Sort value
/// - `transform.t.x`: translation, X axis
/// - `transform.t.y`: translation, Y axis
/// - `transform.t.z`: translation, Z axis
/// - `transform.r.x`: rotation around X axis
/// - `transform.r.y`: rotation around Y axis
/// - `transform.r.z`: rotation around Z axis
/// - `transform.s.x`: scale, X axis
/// - `transform.s.y`: scale, Y axis
/// - `deform`: mesh deformation
#[derive(Debug, Serialize, Deserialize)]
pub struct ParamBinding {
    node: Uuid,
    param_name: String,
    values: Vec<Vec<ParamValue>>,
    #[serde(rename = "isSet")]
    is_set: Vec<Vec<bool>>,
    interpolate_mode: InterpolateMode,
}

impl ParamBinding {
    /// The ID of the node whose property is affected.
    pub fn node(&self) -> Uuid {
        self.node
    }

    pub fn set_node(&mut self, node: Uuid) {
        self.node = node;
    }

    /// The name of the affected node property.
    pub fn param_name(&self) -> &str {
        &self.param_name
    }

    pub fn set_param_name(&mut self, param_name: String) {
        self.param_name = param_name;
    }

    pub fn interpolate_mode(&self) -> InterpolateMode {
        self.interpolate_mode
    }

    pub fn set_interpolate_mode(&mut self, mode: InterpolateMode) {
        self.interpolate_mode = mode;
    }

    // FIXME: offer a better API for these

    /// Returns the values of the bound node property for each axis point.
    ///
    /// This is a 2D array. The first axis is the index of the axis point in Y direction, the second
    /// is the index of the axis point in X direction.
    pub fn values(&self) -> &[Vec<ParamValue>] {
        &self.values
    }

    pub fn set_values(&mut self, values: Vec<Vec<ParamValue>>) {
        self.values = values;
    }

    pub fn is_set(&self) -> &[Vec<bool>] {
        &self.is_set
    }
}

/// A value on the grid of a [`Param`].
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParamValue {
    Scalar(f32),
    Deformation(Vec<Vec2>),
}

impl fmt::Debug for ParamValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scalar(s) => write!(f, "{s}"),
            Self::Deformation(_) => f.write_str("Deformation(…)"),
        }
    }
}

/// Describes how to interpolate between parameter values in a [`ParamBinding`].
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InterpolateMode {
    /// Choose the nearest parameter value.
    Nearest,
    /// Linearly interpolate between the nearest parameter values.
    Linear,
}
