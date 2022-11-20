use std::{
    cmp,
    collections::HashMap,
    str::FromStr,
    sync::{atomic::Ordering, Arc},
};

use rhino2d_io::{InterpolateMode, Uuid};

use crate::{
    atomic::{AtomicF32, AtomicF32x2},
    ord::{is_sorted, TotalF32},
    Error, Result,
};

pub struct ParamMap {
    map: HashMap<Uuid, Vec<ParamBinding>>,
}

impl ParamMap {
    pub(crate) fn lower(io: &[rhino2d_io::Param]) -> Result<Self> {
        let mut map: HashMap<_, Vec<_>> = HashMap::new();
        for param in io {
            let handle = if param.is_vec2() {
                ParamHandle::Param2D(ParamHandle2D {
                    rc: Arc::new(Param2D {
                        axes: [ParamAxis::lower(param, 0)?, ParamAxis::lower(param, 1)?],
                        value: AtomicF32x2::new(param.defaults()[0], param.defaults()[1]),
                    }),
                })
            } else {
                ParamHandle::Param1D(ParamHandle1D {
                    rc: Arc::new(Param1D {
                        axes: [ParamAxis::lower(param, 0)?],
                        value: AtomicF32::new(param.defaults()[0]),
                    }),
                })
            };

            for binding in param.bindings() {
                if binding.interpolate_mode() != InterpolateMode::Linear {
                    return Err(Error::unsupported(format!(
                        "parameter binding interpolation mode '{:?}'",
                        binding.interpolate_mode()
                    )));
                }

                map.entry(binding.node()).or_default().push(ParamBinding {
                    param: handle.clone(),
                    target: ParamTarget::from_str(binding.param_name())?,
                    values: binding
                        .values()
                        .iter()
                        .map(|val| {
                            val.iter()
                                .map(|value| match value {
                                    rhino2d_io::ParamValue::Scalar(f) => Ok(*f),
                                    rhino2d_io::ParamValue::Deformation(_) => {
                                        Err(Error::unsupported("mesh deformation"))
                                    }
                                })
                                .collect::<Result<Vec<_>>>()
                        })
                        .collect::<Result<Vec<_>>>()?,
                });
            }
        }

        Ok(Self { map })
    }

    pub(crate) fn take_params_affecting_node(&mut self, node: Uuid) -> Vec<ParamBinding> {
        self.map.remove(&node).unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
enum ParamHandle {
    Param1D(ParamHandle1D),
    Param2D(ParamHandle2D),
}

#[derive(Debug)]
struct Param1D {
    axes: [ParamAxis; 1],
    value: AtomicF32,
}

#[derive(Debug)]
struct Param2D {
    axes: [ParamAxis; 2],
    value: AtomicF32x2,
}

/// Configuration of a single axis of a parameter.
#[derive(Debug)]
pub struct ParamAxis {
    min: f32,
    max: f32,
    axis_points: Vec<f32>,
}

impl ParamAxis {
    fn lower(param: &rhino2d_io::Param, index: usize) -> Result<Self> {
        let axis_points = param.axis_points()[index].clone();
        if axis_points.is_empty() {
            return Err(Error::invalid(format!(
                "parameter '{}' is invalid: no axis points",
                param.name()
            )));
        }
        if axis_points.first() != Some(&0.0) || axis_points.last() != Some(&1.0) {
            return Err(Error::invalid(format!(
                "parameter '{}' is invalid: invalid axis points ({:?}), first must be 0.0, last must be 1.0",
                param.name(),
                axis_points,
            )));
        }
        if !is_sorted(axis_points.iter().copied().map(TotalF32)) {
            return Err(Error::invalid(format!(
                "axis points of parameter '{}' are not in sorted order: {:?}",
                param.name(),
                axis_points
            )));
        }
        let min = param.min()[index];
        let max = param.max()[index];
        if min > max {
            return Err(Error::invalid(format!(
                "parameter '{}' is invalid: minimum {} is greater than the maximum {}",
                param.name(),
                min,
                max,
            )));
        }
        Ok(Self {
            min,
            max,
            axis_points,
        })
    }

    fn interp(&self, value: f32) -> Interp {
        // clamp and map input value to 0..1, since that's where axis points are defined in
        let value = (value.min(self.max).max(self.min) - self.min) / (self.max - self.min);

        let larger_idx = self
            .axis_points
            .iter()
            .position(|p| p > &value)
            .unwrap_or(self.axis_points.len() - 1);
        let smaller_idx = larger_idx.saturating_sub(1);

        let larger_val = self.axis_points[larger_idx];
        let smaller_val = self.axis_points[smaller_idx];
        let interp = (value - smaller_val) / (larger_val - smaller_val);

        Interp {
            start_index: smaller_idx,
            dist: interp,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Interp {
    start_index: usize,
    /// Value from 0 to 1, 0 means == start, 1 means == start+1
    dist: f32,
}

impl Interp {
    fn lookup(&self, values: &[f32]) -> f32 {
        let start = values[self.start_index];
        if self.dist > 0.0 {
            let end = values[self.start_index + 1];
            start * (1.0 - self.dist) + end * self.dist
        } else {
            start
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParamHandle1D {
    rc: Arc<Param1D>,
}

impl ParamHandle1D {
    pub fn set(&self, value: f32) {
        self.rc.value.store(value, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone)]
pub struct ParamHandle2D {
    rc: Arc<Param2D>,
}

impl ParamHandle2D {
    pub fn set(&self, x: f32, y: f32) {
        self.rc.value.store(x, y, Ordering::Relaxed);
    }
}

/// Describes to a node how a parameter affects one of its properties.
#[derive(Debug, Clone)]
pub struct ParamBinding {
    param: ParamHandle,
    target: ParamTarget,
    values: Vec<Vec<f32>>,
}

impl ParamBinding {
    pub fn value(&self) -> f32 {
        let [x, y] = match &self.param {
            ParamHandle::Param1D(p) => {
                let x = p.rc.value.load(Ordering::Relaxed);
                [
                    p.rc.axes[0].interp(x),
                    Interp {
                        start_index: 0,
                        dist: 0.0,
                    },
                ]
            }
            ParamHandle::Param2D(p) => {
                let [x, y] = p.rc.value.load(Ordering::Relaxed);
                [p.rc.axes[0].interp(x), p.rc.axes[1].interp(y)]
            }
        };

        // TODO `InterpolateMode::Nearest`

        let start_row = &self.values[y.start_index];
        let start = x.lookup(start_row);
        if y.dist > 0.0 {
            let end_row = &self.values[cmp::min(y.start_index + 1, self.values.len() - 1)];
            let end = x.lookup(end_row);
            start * (1.0 - y.dist) + end * y.dist
        } else {
            start
        }
    }

    pub fn target(&self) -> ParamTarget {
        self.target
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamTarget {
    ZSort,
    TranslationX,
    TranslationY,
    TranslationZ,
    RotationX,
    RotationY,
    RotationZ,
    ScaleX,
    ScaleY,
}

impl FromStr for ParamTarget {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "zSort" => Self::ZSort,
            "transform.t.x" => Self::TranslationX,
            "transform.t.y" => Self::TranslationY,
            "transform.t.z" => Self::TranslationZ,
            "transform.r.x" => Self::RotationX,
            "transform.r.y" => Self::RotationY,
            "transform.r.z" => Self::RotationZ,
            "transform.s.x" => Self::ScaleX,
            "transform.s.y" => Self::ScaleY,
            _ => {
                return Err(Error::unsupported(format!("parameter target '{}'", s)));
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_axis_interp() {
        // Axis points at -1.0, 0.0, and 1.0
        let axis = ParamAxis {
            min: -1.0,
            max: 1.0,
            axis_points: vec![0.0, 0.5, 1.0],
        };
        assert_eq!(
            axis.interp(-100.0),
            Interp {
                start_index: 0,
                dist: 0.0
            }
        );
        assert_eq!(
            axis.interp(-1.000001),
            Interp {
                start_index: 0,
                dist: 0.0
            }
        );
        assert_eq!(
            axis.interp(-1.0),
            Interp {
                start_index: 0,
                dist: 0.0
            }
        );
        assert_eq!(
            axis.interp(-0.5),
            Interp {
                start_index: 0,
                dist: 0.5
            }
        );
        assert_eq!(
            axis.interp(0.0),
            Interp {
                start_index: 1,
                dist: 0.0
            }
        );
        assert_eq!(
            axis.interp(0.5),
            Interp {
                start_index: 1,
                dist: 0.5
            }
        );
        assert_eq!(
            axis.interp(1.0),
            Interp {
                start_index: 1,
                dist: 1.0
            }
        );
        assert_eq!(
            axis.interp(1.0000001),
            Interp {
                start_index: 1,
                dist: 1.0
            }
        );
        assert_eq!(
            axis.interp(100.0),
            Interp {
                start_index: 1,
                dist: 1.0
            }
        );
    }

    #[test]
    fn test_interp_lookup() {
        assert_eq!(
            Interp {
                start_index: 0,
                dist: 0.0
            }
            .lookup(&[0.0]),
            0.0
        );
        assert_eq!(
            Interp {
                start_index: 0,
                dist: 0.5
            }
            .lookup(&[0.0, 1.0]),
            0.5
        );
        assert_eq!(
            Interp {
                start_index: 0,
                dist: 1.0,
            }
            .lookup(&[0.0, 1.0]),
            1.0
        );
        assert_eq!(
            Interp {
                start_index: 1,
                dist: 0.0
            }
            .lookup(&[0.0, 1.0]),
            1.0
        );
        assert_eq!(
            Interp {
                start_index: 1,
                dist: 0.5
            }
            .lookup(&[0.0, 1.0, 2.0]),
            1.5
        );
        assert_eq!(
            Interp {
                start_index: 1,
                dist: 0.25
            }
            .lookup(&[0.0, 1.0, 2.0]),
            1.25
        );
        assert_eq!(
            Interp {
                start_index: 1,
                dist: 0.75
            }
            .lookup(&[0.0, 1.0, 2.0]),
            1.75
        );
    }
}
