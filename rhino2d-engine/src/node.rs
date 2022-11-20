//! Node representation for the puppeteering engine.

use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Mul;
use std::time::Duration;

use nalgebra::Matrix4;
use nalgebra::Vector3;
use rhino2d_io::node as io_node;
use rhino2d_io::Uuid;

use crate::param::ParamBinding;
use crate::param::ParamMap;
use crate::param::ParamTarget;
use crate::RenderBuffer;
use crate::RenderCommand;
use crate::Result;

pub enum Node {
    /// Hierarchy-only node that isn't visible.
    Node(NodeBase),
    Drawable(Drawable),
}

impl Deref for Node {
    type Target = NodeBase;

    fn deref(&self) -> &Self::Target {
        match self {
            Node::Node(node) => node,
            Node::Drawable(node) => node,
        }
    }
}

impl DerefMut for Node {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Node::Node(node) => node,
            Node::Drawable(node) => node,
        }
    }
}

impl Node {
    pub(crate) fn from_io(params: &mut ParamMap, io: &io_node::Node) -> Result<Self> {
        match io {
            io_node::Node::Node(node) => Ok(Self::Node(NodeBase::from_io(params, node)?)),
            io_node::Node::Drawable(node) => Ok(Self::Drawable(Drawable::from_io(params, node)?)),
            io_node::Node::Part(node) => Ok(Self::Drawable(Drawable::from_io(params, node)?)),
            _ => Err(crate::Error::unsupported(format!(
                "node '{}' has unimplemented node type '{:?}'",
                io.name(),
                io
            ))),
        }
    }

    pub(crate) fn update(&mut self, delta: Duration, rbuf: &mut RenderBuffer) {
        let root_transform = Transform::identity();
        self.update_recursive(delta, rbuf, &root_transform);
    }
}

pub struct NodeBase {
    uuid: Uuid,
    children: Vec<Node>,
    /// List of parameter bindings that affect this node.
    params: Vec<ParamBinding>,

    /// Transform of this node, as specified by the model.
    ///
    /// Relative to the parent node, without any parameter offsets applied.
    base_transform: Transform,
    /// Z-Sort order from the model.
    base_zsort: f32,

    global_transform: Transform,
    zsort: f32,
    /// Ignores the parent node's transform.
    lock_to_root: bool,
}

impl NodeBase {
    fn from_io(params: &mut ParamMap, io: &io_node::NodeBase) -> Result<Self> {
        Ok(Self {
            uuid: io.uuid(),
            children: io
                .children()
                .iter()
                .map(|ch| Node::from_io(params, ch))
                .collect::<Result<_>>()?,
            params: params.take_params_affecting_node(io.uuid()),
            base_transform: Transform::from_io(io.transform()),
            base_zsort: io.zsort(),
            global_transform: Transform::identity(),
            zsort: io.zsort(),
            lock_to_root: io.lock_to_root(),
        })
    }

    /// Updates `self`'s `global_transform` and `zsort` values based on `parent_transform` and
    /// parameters affecting `self`.
    fn update_self(&mut self, rbuf: &mut RenderBuffer, parent_transform: &Transform) {
        // Parameters need to be applied to the base transform first (eg. rotation applies to the
        // node's origin, not the whole model's origin).
        let mut zsort = self.base_zsort;
        let mut param_tf = rhino2d_io::node::Transform::new();

        for param in &self.params {
            let value = param.value();
            match param.target() {
                ParamTarget::ZSort => zsort += value,
                ParamTarget::TranslationX => param_tf.translation_mut()[0] += value,
                ParamTarget::TranslationY => param_tf.translation_mut()[1] += value,
                ParamTarget::TranslationZ => param_tf.translation_mut()[2] += value,
                ParamTarget::RotationX => param_tf.rotation_mut()[0] += value,
                ParamTarget::RotationY => param_tf.rotation_mut()[1] += value,
                ParamTarget::RotationZ => param_tf.rotation_mut()[2] += value,
                ParamTarget::ScaleX => param_tf.scale_mut()[0] += value,
                ParamTarget::ScaleY => param_tf.scale_mut()[1] += value,
            }
        }

        let self_transform = self.base_transform * Transform::from_io(&param_tf);

        self.zsort = zsort;
        if self.lock_to_root {
            self.global_transform = self_transform;
        } else {
            self.global_transform = self_transform * *parent_transform;
        }

        rbuf.push(RenderCommand {
            node: self.uuid,
            transform: self.global_transform,
            zsort,
            deform: None,
        });
    }

    /// Updates `self`'s transform/zsort and all child nodes, recursively.
    fn update_recursive(
        &mut self,
        delta: Duration,
        rbuf: &mut RenderBuffer,
        parent_transform: &Transform,
    ) {
        self.update_self(rbuf, parent_transform);

        for child in &mut self.children {
            child.update_recursive(delta, rbuf, &self.global_transform);
        }
    }
}

pub struct Drawable {
    node: NodeBase,
}

impl Deref for Drawable {
    type Target = NodeBase;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl DerefMut for Drawable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

impl Drawable {
    fn from_io(params: &mut ParamMap, io: &io_node::Drawable) -> Result<Self> {
        Ok(Self {
            node: NodeBase::from_io(params, io)?,
        })
    }
}

/// An affine transformation, represented as a 4x4 matrix of `f32` values.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    mat: Matrix4<f32>,
}

impl Transform {
    pub(crate) fn identity() -> Self {
        Self {
            mat: Matrix4::identity(),
        }
    }

    /// Converts an `inochi_io` transform to an `inochi_engine` transform.
    pub(crate) fn from_io(t: &rhino2d_io::node::Transform) -> Self {
        let rot = t.rotation();
        let scale = t.scale();
        let trans = t.translation();
        Self {
            mat: Matrix4::new_nonuniform_scaling(&Vector3::new(scale[0], scale[1], 1.0))
                * Matrix4::from_euler_angles(rot[0], rot[1], rot[2])
                * Matrix4::new_translation(&Vector3::new(trans[0], trans[1], trans[2])),
        }
    }

    /// Returns the raw matrix data, in column-major order.
    pub fn as_column_major_data(&self) -> &[f32] {
        self.mat.as_slice()
    }
}

impl Mul for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mat = self.mat * rhs.mat;
        Self { mat }
    }
}
