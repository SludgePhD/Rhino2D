use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::Vec2;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Automation {
    Sine(SineAutomation),
    Physics(PhysicsAutomation),
}

impl Deref for Automation {
    type Target = AutomationBase;

    fn deref(&self) -> &Self::Target {
        match self {
            Automation::Sine(a) => a,
            Automation::Physics(a) => a,
        }
    }
}

impl DerefMut for Automation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Automation::Sine(a) => a,
            Automation::Physics(a) => a,
        }
    }
}

impl From<SineAutomation> for Automation {
    fn from(a: SineAutomation) -> Self {
        Self::Sine(a)
    }
}

impl From<PhysicsAutomation> for Automation {
    fn from(a: PhysicsAutomation) -> Self {
        Self::Physics(a)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutomationBase {
    name: String,
    bindings: Vec<AutomationBinding>,
}

impl AutomationBase {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn bindings(&self) -> &[AutomationBinding] {
        &self.bindings
    }

    pub fn bindings_mut(&mut self) -> &mut [AutomationBinding] {
        &mut self.bindings
    }

    pub fn push_binding(&mut self, binding: AutomationBinding) {
        self.bindings.push(binding);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SineAutomation {
    #[serde(flatten)]
    automation: AutomationBase,
    speed: f32,
    sine_type: SineType,
}

impl SineAutomation {
    pub fn new(name: String, sine_type: SineType, speed: f32) -> Self {
        Self {
            automation: AutomationBase {
                name,
                bindings: Vec::new(),
            },
            speed,
            sine_type,
        }
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn sine_type(&self) -> SineType {
        self.sine_type
    }

    pub fn set_sine_type(&mut self, sine_type: SineType) {
        self.sine_type = sine_type;
    }
}

// Some wicked souls claim that using `Deref` to emulate inheritance is Bad™. These people are paid
// by the pharma industry to give people RSI by making them write even more boilerplate than Rust
// already requires by itself. Don't listen to their lies.
impl Deref for SineAutomation {
    type Target = AutomationBase;

    fn deref(&self) -> &Self::Target {
        &self.automation
    }
}

impl DerefMut for SineAutomation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.automation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
#[non_exhaustive]
pub enum SineType {
    Sin = 0,
    Cos = 1,
    Tan = 2,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PhysicsAutomation {
    #[serde(flatten)]
    automation: AutomationBase,
    nodes: Vec<VerletNode>,
    damping: f32,
    bounciness: f32,
    gravity: f32,
}

impl PhysicsAutomation {
    pub fn new(name: String) -> Self {
        Self {
            automation: AutomationBase {
                name,
                bindings: Vec::new(),
            },
            nodes: Vec::new(),
            damping: 0.05,
            bounciness: 1.0,
            gravity: 20.0,
        }
    }

    pub fn nodes(&self) -> &[VerletNode] {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut [VerletNode] {
        &mut self.nodes
    }

    pub fn push_node(&mut self, node: VerletNode) {
        self.nodes.push(node);
    }

    pub fn damping(&self) -> f32 {
        self.damping
    }

    pub fn set_damping(&mut self, damping: f32) {
        self.damping = damping;
    }

    pub fn bounciness(&self) -> f32 {
        self.bounciness
    }

    pub fn set_bounciness(&mut self, bounciness: f32) {
        self.bounciness = bounciness;
    }

    pub fn gravity(&self) -> f32 {
        self.gravity
    }

    pub fn set_gravity(&mut self, gravity: f32) {
        self.gravity = gravity;
    }
}

impl Deref for PhysicsAutomation {
    type Target = AutomationBase;

    fn deref(&self) -> &Self::Target {
        &self.automation
    }
}

impl DerefMut for PhysicsAutomation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.automation
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerletNode {
    distance: f32,
    position: Vec2,
    old_position: Vec2,
}

impl VerletNode {
    pub fn new(position: Vec2) -> Self {
        Self {
            distance: 1.0,
            position,
            old_position: position,
        }
    }

    /// Returns the node's desired distance to its parent.
    pub fn distance(&self) -> f32 {
        self.distance
    }

    pub fn set_distance(&mut self, distance: f32) {
        self.distance = distance;
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    pub fn old_position(&self) -> Vec2 {
        self.old_position
    }
}

/// Describes how an [`Automation`] affects a [`Param`] of the model.
///
/// [`Param`]: crate::Param
#[derive(Debug, Serialize, Deserialize)]
pub struct AutomationBinding {
    param: String,
    axis: AutomationAxis,
    range: Vec2,
}

impl AutomationBinding {
    pub fn param(&self) -> &str {
        &self.param
    }

    /// Sets the name of the [`Param`][crate::Param] affected by this binding.
    ///
    /// FIXME: why does this not use the UUID?
    pub fn set_param(&mut self, param: String) {
        self.param = param;
    }

    pub fn axis(&self) -> AutomationAxis {
        self.axis
    }

    /// Sets the axis of the [`Param`][crate::Param] that is affected.
    pub fn set_axis(&mut self, axis: AutomationAxis) {
        self.axis = axis;
    }

    pub fn range(&self) -> Vec2 {
        self.range
    }

    pub fn set_range(&mut self, range: Vec2) {
        self.range = range;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum AutomationAxis {
    X = 0,
    Y = 1,
}
