//! Model node definitions.
//!
//! An Inochi2D model has a root node ([`InochiPuppet::root_node`]), and each node can have an
//! arbitrary number of child nodes, forming a tree. A node's position, rotation, and scale is
//! relative to its parent.
//!
//! A nodes position in the tree does not affect its visibility, masking, or draw order relative to
//! other nodes.
//!
//! [`InochiPuppet::root_node`]: crate::InochiPuppet::root_node

use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{Uuid, Vec2, Vec3};

/// Enumeration of all supported node types.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Node {
    Node(NodeBase),
    Drawable(Drawable),

    PathDeform(PathDeform),
    Part(Part),
    Mask(Mask),
    Composite(Composite),
    SimplePhysics(SimplePhysics),
}

impl Node {
    pub fn type_name(&self) -> &str {
        match self {
            Node::Node(_) => "Node",
            Node::Drawable(_) => "Drawable",
            Node::PathDeform(_) => "PathDeform",
            Node::Part(_) => "Part",
            Node::Mask(_) => "Mask",
            Node::Composite(_) => "Composite",
            Node::SimplePhysics(_) => "SimplePhysics",
        }
    }
}

impl Deref for Node {
    type Target = NodeBase;

    fn deref(&self) -> &Self::Target {
        match self {
            Node::Node(n) => n,
            Node::Drawable(n) => n,
            Node::PathDeform(n) => n,
            Node::Part(n) => n,
            Node::Mask(n) => n,
            Node::Composite(n) => n,
            Node::SimplePhysics(n) => n,
        }
    }
}

impl DerefMut for Node {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Node::Node(n) => n,
            Node::Drawable(n) => n,
            Node::PathDeform(n) => n,
            Node::Part(n) => n,
            Node::Mask(n) => n,
            Node::Composite(n) => n,
            Node::SimplePhysics(n) => n,
        }
    }
}

/// Base type shared by all nodes.
///
/// All node types in this model [`Deref`] to this base type and have its properties.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeBase {
    uuid: Uuid,
    name: String,
    enabled: bool,
    zsort: f32,
    transform: Transform,
    lock_to_root: bool,
    children: Option<Vec<Node>>,
}

impl NodeBase {
    pub fn new(uuid: Uuid, name: String) -> Self {
        Self {
            uuid,
            name,
            enabled: true,
            zsort: 0.0,
            transform: Transform::new(),
            lock_to_root: false,
            children: None,
        }
    }

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

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns the node's Z-Sort value.
    ///
    /// The Z-Axis points into the scene, so nodes with a *lower* Z-Sort value are in front of
    /// nodes with a higher Z-Sort value.
    pub fn zsort(&self) -> f32 {
        self.zsort
    }

    pub fn set_zsort(&mut self, zsort: f32) {
        self.zsort = zsort;
    }

    /// Returns the node's transformation relative to its parent.
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    pub fn lock_to_root(&self) -> bool {
        self.lock_to_root
    }

    pub fn set_lock_to_root(&mut self, lock_to_root: bool) {
        self.lock_to_root = lock_to_root;
    }

    pub fn children(&self) -> &[Node] {
        self.children.as_deref().unwrap_or(&[])
    }

    pub fn children_mut(&mut self) -> &mut [Node] {
        self.children.as_deref_mut().unwrap_or(&mut [])
    }

    pub fn push_child(&mut self, node: Node) {
        self.children.get_or_insert(Vec::new()).push(node);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Composite {
    #[serde(flatten)]
    node: NodeBase,
    blend_mode: BlendMode,
    tint: Vec3,
    mask_threshold: f32,
    opacity: f32,
}

impl Composite {
    pub fn blend_mode(&self) -> BlendMode {
        self.blend_mode
    }

    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }

    pub fn tint(&self) -> Vec3 {
        self.tint
    }

    pub fn set_tint(&mut self, tint: Vec3) {
        self.tint = tint;
    }

    pub fn mask_threshold(&self) -> f32 {
        self.mask_threshold
    }

    pub fn set_mask_threshold(&mut self, mask_threshold: f32) {
        self.mask_threshold = mask_threshold;
    }

    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
    }
}

impl Deref for Composite {
    type Target = NodeBase;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl DerefMut for Composite {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

/// Describes how to blend a node onto the pixels below it.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BlendMode {
    /// Perform standard alpha blending.
    Normal,
    Multiply,
    ColorDodge,
    LinearDodge,
    Screen,
    ClipToLower,
    SliceFromLower,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathDeform {
    #[serde(flatten)]
    node: NodeBase,
    joints: Vec<Vec2>,
    bindings: Vec<JointBindingData>,
}

impl PathDeform {
    pub fn joint_origins(&self) -> &[Vec2] {
        &self.joints
    }

    pub fn set_joint_origins(&mut self, joints: Vec<Vec2>) {
        self.joints = joints;
    }

    pub fn push_joint(&mut self, origin: Vec2) {
        self.joints.push(origin);
    }

    pub fn bindings(&self) -> &[JointBindingData] {
        &self.bindings
    }

    pub fn bindings_mut(&mut self) -> &mut [JointBindingData] {
        &mut self.bindings
    }

    pub fn set_bindings(&mut self, bindings: Vec<JointBindingData>) {
        self.bindings = bindings;
    }

    pub fn push_binding(&mut self, binding: JointBindingData) {
        self.bindings.push(binding);
    }
}

impl Deref for PathDeform {
    type Target = NodeBase;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl DerefMut for PathDeform {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

/// Describes how a [`Drawable`] is affected by a list of joints.
///
/// There is one [`JointBindingData`] object per [`Drawable`] affected by a [`PathDeform`].
#[derive(Debug, Serialize, Deserialize)]
pub struct JointBindingData {
    bound_to: Uuid,
    bind_data: Vec<Vec<usize>>,
}

impl JointBindingData {
    /// Returns the ID of the [`Drawable`] this binding data attaches joints to.
    pub fn bound_to(&self) -> Uuid {
        self.bound_to
    }

    pub fn set_bound_to(&mut self, bound_to: Uuid) {
        self.bound_to = bound_to;
    }

    /// Returns the binding data for the attached [`Drawable`].
    ///
    /// Every entry in the returned slice corresponds to one joint. Every entry in the contained
    /// `Vec<usize>` is a vertex index that should be affected by the joint.
    ///
    /// FIXME: provide a better interface, this is really hard to understand
    pub fn bind_data(&self) -> &[Vec<usize>] {
        &self.bind_data
    }
}

/// A node with associated mesh data.
#[derive(Debug, Serialize, Deserialize)]
pub struct Drawable {
    #[serde(flatten)]
    node: NodeBase,
    mesh: MeshData,
}

impl Drawable {
    pub fn mesh_data(&self) -> &MeshData {
        &self.mesh
    }

    pub fn mesh_data_mut(&mut self) -> &mut MeshData {
        &mut self.mesh
    }

    pub fn set_mesh_data(&mut self, mesh_data: MeshData) {
        self.mesh = mesh_data;
    }
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

/// A rendered "part" of the model.
///
/// This node type is typically used for all visual components of the model.
#[derive(Debug, Serialize, Deserialize)]
pub struct Part {
    #[serde(flatten)]
    drawable: Drawable,
    textures: Vec<u32>,
    opacity: f32,
    mask_threshold: f32,
    tint: Vec3,
    blend_mode: BlendMode,
    mask_mode: Option<MaskMode>,
    masked_by: Option<Vec<Uuid>>,
}

impl Part {
    /// Returns the texture indices used by this [`Part`].
    ///
    /// Texture indices index into [`crate::InochiPuppet::textures`]. Currently, parts can only use
    /// a single texture.
    pub fn textures(&self) -> &[u32] {
        &self.textures
    }

    pub fn set_textures(&mut self, textures: Vec<u32>) {
        self.textures = textures;
    }

    pub fn push_texture(&mut self, texture: u32) {
        self.textures.push(texture);
    }

    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
    }

    pub fn mask_threshold(&self) -> f32 {
        self.mask_threshold
    }

    pub fn set_mask_threshold(&mut self, thresh: f32) {
        self.mask_threshold = thresh;
    }

    pub fn tint(&self) -> Vec3 {
        self.tint
    }

    pub fn set_tint(&mut self, tint: Vec3) {
        self.tint = tint;
    }

    pub fn blend_mode(&self) -> BlendMode {
        self.blend_mode
    }

    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }

    pub fn mask_mode(&self) -> Option<MaskMode> {
        self.mask_mode
    }

    pub fn set_mask_mode(&mut self, mode: Option<MaskMode>) {
        self.mask_mode = mode;
    }

    /// Returns the IDs of the [`Drawable`]s that mask `self`.
    pub fn masked_by(&self) -> &[Uuid] {
        self.masked_by.as_deref().unwrap_or(&[])
    }

    pub fn set_masked_by(&mut self, masked_by: Option<Vec<Uuid>>) {
        self.masked_by = masked_by;
    }
}

impl Deref for Part {
    type Target = Drawable;

    fn deref(&self) -> &Self::Target {
        &self.drawable
    }
}

impl DerefMut for Part {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.drawable
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mask {
    #[serde(flatten)]
    drawable: Drawable,
}

impl Deref for Mask {
    type Target = Drawable;

    fn deref(&self) -> &Self::Target {
        &self.drawable
    }
}

impl DerefMut for Mask {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.drawable
    }
}

/// Specifies how mask sources affect the [`Part`] they are applied to.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MaskMode {
    /// The part's pixels are restricted to areas where the mask layer is also present.
    Mask,
    /// The part's pixels are restricted to areas where the mask layer is *not* present.
    Dodge,
}

/// Triangle mesh data underlying all [`Drawable`] nodes.
#[derive(Debug, Serialize, Deserialize)]
pub struct MeshData {
    verts: Vec<f32>,
    uvs: Option<Vec<f32>>,
    indices: Vec<u16>,
    origin: Vec2,
}

impl MeshData {
    pub fn verts(&self) -> impl Iterator<Item = Vec2> + '_ {
        self.verts.chunks(2).map(|chunk| [chunk[0], chunk[1]])
    }

    pub fn uvs(&self) -> Option<impl Iterator<Item = Vec2> + '_> {
        Some(
            self.uvs
                .as_ref()?
                .chunks(2)
                .map(|chunk| [chunk[0], chunk[1]]),
        )
    }

    pub fn indices(&self) -> &[u16] {
        &self.indices
    }

    pub fn vertex_count(&self) -> usize {
        self.verts.len() / 2
    }

    pub fn origin(&self) -> Vec2 {
        self.origin
    }

    pub fn set_origin(&mut self, origin: Vec2) {
        self.origin = origin;
    }
}

/// An affine transformation.
///
/// Scale is applied first, then rotation, then translation.
///
/// Y points down, X to the right, Z points into the scene.
#[derive(Debug, Serialize, Deserialize)]
pub struct Transform {
    trans: Vec3,
    rot: Vec3,
    scale: Vec2,
}

impl Transform {
    /// Creates an identity transform.
    pub fn new() -> Self {
        Self {
            trans: [0.0; 3],
            rot: [0.0; 3],
            scale: [1.0; 2],
        }
    }

    pub fn translation(&self) -> Vec3 {
        self.trans
    }

    pub fn translation_mut(&mut self) -> &mut Vec3 {
        &mut self.trans
    }

    /// Sets the transform's translation.
    ///
    /// Note that translation allows for 3 dimentions, but translation in Z direction does *not*
    /// affect the Z-Sort order.
    ///
    /// FIXME: what does Z translation *do*?
    pub fn set_translation(&mut self, translation: Vec3) {
        self.trans = translation;
    }

    pub fn rotation(&self) -> Vec3 {
        self.rot
    }

    pub fn rotation_mut(&mut self) -> &mut Vec3 {
        &mut self.rot
    }

    pub fn set_rotation(&mut self, rotation: Vec3) {
        self.rot = rotation;
    }

    pub fn scale(&self) -> Vec2 {
        self.scale
    }

    pub fn scale_mut(&mut self) -> &mut Vec2 {
        &mut self.scale
    }

    pub fn set_scale(&mut self, scale: Vec2) {
        self.scale = scale;
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimplePhysics {
    #[serde(flatten)]
    node: NodeBase,
    param: Uuid,
    model_type: PhysicsModel,
    map_mode: ParamMapMode,
    gravity: f32,
    length: f32,
    frequency: f32,
    angle_damping: f32,
    length_damping: f32,
    output_scale: Vec2,
}

impl SimplePhysics {
    /// Returns the parameter ID this physics object is bound to.
    ///
    /// If not bound, an ID of `-1` is used.
    pub fn param(&self) -> Uuid {
        self.param
    }

    pub fn set_param(&mut self, param: Uuid) {
        self.param = param;
    }

    pub fn model_type(&self) -> PhysicsModel {
        self.model_type
    }

    pub fn set_model_type(&mut self, model_type: PhysicsModel) {
        self.model_type = model_type;
    }

    pub fn map_mode(&self) -> ParamMapMode {
        self.map_mode
    }

    pub fn set_map_mode(&mut self, map_mode: ParamMapMode) {
        self.map_mode = map_mode;
    }

    pub fn gravity(&self) -> f32 {
        self.gravity
    }

    pub fn set_gravity(&mut self, gravity: f32) {
        self.gravity = gravity;
    }

    pub fn length(&self) -> f32 {
        self.length
    }

    pub fn set_length(&mut self, length: f32) {
        self.length = length;
    }

    pub fn frequency(&self) -> f32 {
        self.frequency
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }

    pub fn angle_damping(&self) -> f32 {
        self.angle_damping
    }

    pub fn set_angle_damping(&mut self, angle_damping: f32) {
        self.angle_damping = angle_damping;
    }

    pub fn length_damping(&self) -> f32 {
        self.length_damping
    }

    pub fn set_length_damping(&mut self, length_damping: f32) {
        self.length_damping = length_damping;
    }

    pub fn output_scale(&self) -> Vec2 {
        self.output_scale
    }

    pub fn set_output_scale(&mut self, output_scale: Vec2) {
        self.output_scale = output_scale;
    }
}

impl Deref for SimplePhysics {
    type Target = NodeBase;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl DerefMut for SimplePhysics {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum PhysicsModel {
    Pendulum,
    SpringPendulum,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum ParamMapMode {
    AngleLength,
    XY,
}
