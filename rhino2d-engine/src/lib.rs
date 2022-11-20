//! Contains code to animate and simulate an Inochi2D model.
//!
//! Note that this crate is not a renderer. It computes which model nodes to render where and in
//! which order, but does not do the rendering itself. That step is delegated to other crates.

pub mod atomic;
pub mod node;
mod ord;
pub mod param;

use core::fmt;
use std::time::Duration;

use node::{Node, Transform};
use ord::TotalF32;
use param::ParamMap;
use rhino2d_io::{Uuid, Vec2};

pub struct RenderCommand {
    node: Uuid,
    zsort: f32,
    transform: Transform,
    deform: Option<Vec<Vec2>>,
}

impl RenderCommand {
    /// Returns the ID of the node to render.
    pub fn node(&self) -> Uuid {
        self.node
    }

    /// Returns the node's computed Z-Sort value.
    pub fn zsort(&self) -> f32 {
        self.zsort
    }

    /// Returns the node's computed global transform.
    pub fn transform(&self) -> Transform {
        self.transform
    }

    /// Returns the node's vertex deformations, if there is any.
    ///
    /// If this returns `Some`, the number of entries in the slice will match the number of vertices
    /// of the node's mesh.
    pub fn deform(&self) -> Option<&[Vec2]> {
        self.deform.as_deref()
    }
}

/// Records rendering commands while nodes are being updated.
struct RenderBuffer {
    commands: Vec<RenderCommand>,
}

impl RenderBuffer {
    fn push(&mut self, cmd: RenderCommand) {
        self.commands.push(cmd);
    }

    fn finish(&mut self) {
        // Sort by Z-Sort value, *de*scending.
        self.commands.sort_by_key(|cmd| TotalF32(-cmd.zsort));

        // Now `commands` has the back-most node in the front, which is the typical render order.
    }
}

pub struct PuppetEngine {
    root_node: Node,
    render_buffer: RenderBuffer,
}

impl PuppetEngine {
    pub fn new(puppet: &rhino2d_io::InochiPuppet) -> Result<Self> {
        let mut param_map = ParamMap::lower(puppet.params())?;
        Ok(Self {
            root_node: Node::from_io(&mut param_map, puppet.root_node())?,
            render_buffer: RenderBuffer {
                commands: Vec::new(),
            },
        })
    }

    pub fn update(&mut self, delta: Duration) -> &[RenderCommand] {
        self.root_node.update(delta, &mut self.render_buffer);

        self.render_buffer.finish();
        &self.render_buffer.commands
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.msg)
    }
}

impl std::error::Error for Error {}

impl Error {
    fn unsupported(what: impl AsRef<str>) -> Self {
        Self {
            msg: format!("model uses unsupported feature: {}", what.as_ref()),
        }
    }

    fn invalid(what: impl AsRef<str>) -> Self {
        Self {
            msg: format!("invalid model: {}", what.as_ref()),
        }
    }
}
