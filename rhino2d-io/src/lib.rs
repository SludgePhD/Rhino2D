pub mod automation;
mod metadata;
pub mod node;
mod param;
mod physics;

use automation::Automation;
pub use metadata::*;
use node::Node;
pub use param::*;
pub use physics::*;

use std::{
    fmt,
    fs::File,
    io::{self, BufReader, BufWriter, Read, Write},
    path::Path,
};

use byteorder::{ReadBytesExt, WriteBytesExt, BE};
use serde::{Deserialize, Serialize};

const MAGIC: [u8; 8] = *b"TRNSRTS\0";
const MAGIC_TEX: [u8; 8] = *b"TEX_SECT";
const MAGIC_EXT: [u8; 8] = *b"EXT_SECT";

/// An Inochi2D puppet.
#[derive(Debug)]
pub struct InochiPuppet {
    data: JsonData,
    textures: Vec<Texture>,
    vendor_data: Vec<VendorData>,
}

impl InochiPuppet {
    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::from_read(&mut BufReader::new(File::open(path.as_ref())?))
    }

    pub fn from_read<R: Read>(read: &mut R) -> io::Result<Self> {
        Self::from_read_impl(read)
    }

    fn from_read_impl(read: &mut dyn Read) -> io::Result<Self> {
        let mut magic = [0; 8];
        read.read_exact(&mut magic)?;
        if magic != MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "invalid magic bytes, expected '{}', got '{}'",
                    MAGIC.escape_ascii(),
                    magic.escape_ascii()
                ),
            ));
        }

        let json_len = read.read_u32::<BE>()?;
        let mut buf = vec![0; json_len as usize];
        read.read_exact(&mut buf)?;
        let mut de = serde_json::Deserializer::from_slice(&buf);
        let json: JsonData = serde_ignored::deserialize(&mut de, |unused| {
            log::warn!("deserializer ignoring `{}`", unused);
        })
        .map_err(|e| {
            log::error!(
                "failed to deserialize; model JSON dump:\n{}",
                String::from_utf8_lossy(&buf),
            );
            e
        })?;

        let mut magic = [0; 8];
        read.read_exact(&mut magic)?;
        if magic != MAGIC_TEX {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "invalid magic bytes for texture section, expected '{}', got '{}'",
                    MAGIC_TEX.escape_ascii(),
                    magic.escape_ascii()
                ),
            ));
        }

        let texture_count = read.read_u32::<BE>()?;
        let mut textures = Vec::with_capacity(texture_count as usize);

        for _ in 0..texture_count {
            let payload_len = read.read_u32::<BE>()?;
            let encoding = read.read_u8()?;
            let encoding = match encoding {
                0 => TextureEncoding::Png,
                1 => TextureEncoding::Tga,
                2 => TextureEncoding::Bc7,
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("invalid texture encoding value {encoding}"),
                    ))
                }
            };

            let mut data = vec![0; payload_len as usize];
            read.read_exact(&mut data)?;

            textures.push(Texture {
                enc: encoding,
                data,
            });
        }

        // Optional EXT Vendor Data section.
        let mut vendor_payloads = Vec::new();
        let mut magic = [0; 8];
        match read.read_exact(&mut magic) {
            Ok(_) => {
                if magic != MAGIC_EXT {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "invalid magic bytes for EXT section, expected '{}', got '{}'",
                            MAGIC_EXT.escape_ascii(),
                            magic.escape_ascii()
                        ),
                    ));
                }

                let payload_count = read.read_u32::<BE>()?;
                vendor_payloads = Vec::with_capacity(payload_count as usize);

                for _ in 0..payload_count {
                    let name_len = read.read_u32::<BE>()?;
                    let mut name = String::with_capacity(name_len as usize);
                    read.take(name_len.into()).read_to_string(&mut name)?;

                    let payload_len = read.read_u32::<BE>()?;
                    let mut data = vec![0; payload_len as usize];
                    read.read_exact(&mut data)?;
                    vendor_payloads.push(VendorData {
                        name,
                        payload: data,
                    });
                }
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {}
            Err(e) => return Err(e),
        }

        Ok(Self {
            data: json,
            textures,
            vendor_data: vendor_payloads,
        })
    }

    /// Writes this model to a file at `path`.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self.write(BufWriter::new(File::create(path.as_ref())?))
    }

    /// Serializes this model into a type that implements [`Write`].
    pub fn write<W: Write>(&self, mut w: W) -> io::Result<()> {
        w.write_all(&MAGIC)?;
        let json = serde_json::to_vec(&self.data)?;
        w.write_u32::<BE>(json.len().try_into().unwrap())?;
        w.write_all(&json)?;

        w.write_all(&MAGIC_TEX)?;
        w.write_u32::<BE>(self.textures().len().try_into().unwrap())?;
        for tex in self.textures() {
            w.write_u32::<BE>(tex.data().len().try_into().unwrap())?;
            w.write_u8(tex.encoding() as u8)?;
            w.write_all(tex.data())?;
        }

        w.write_all(&MAGIC_EXT)?;
        w.write_u32::<BE>(self.vendor_data().len().try_into().unwrap())?;
        for data in self.vendor_data() {
            w.write_u32::<BE>(data.name().len().try_into().unwrap())?;
            w.write_all(data.name().as_bytes())?;
            w.write_u32::<BE>(data.payload().len().try_into().unwrap())?;
            w.write_all(data.payload())?;
        }

        Ok(())
    }

    /// Returns a reference to the model metadata, containing author, license, and version
    /// information.
    pub fn metadata(&self) -> &Metadata {
        &self.data.meta
    }

    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.data.meta
    }

    pub fn set_metadata(&mut self, metadata: Metadata) {
        self.data.meta = metadata;
    }

    pub fn physics(&self) -> &Physics {
        &self.data.physics
    }

    pub fn physics_mut(&mut self) -> &mut Physics {
        &mut self.data.physics
    }

    pub fn set_physics(&mut self, physics: Physics) {
        self.data.physics = physics;
    }

    pub fn root_node(&self) -> &Node {
        &self.data.nodes
    }

    pub fn root_node_mut(&mut self) -> &mut Node {
        &mut self.data.nodes
    }

    pub fn set_root_node(&mut self, node: Node) {
        self.data.nodes = node;
    }

    pub fn params(&self) -> &[Param] {
        &self.data.param
    }

    pub fn params_mut(&mut self) -> &mut [Param] {
        &mut self.data.param
    }

    pub fn push_param(&mut self, param: Param) {
        self.data.param.push(param);
    }

    pub fn automations(&self) -> &[Automation] {
        self.data.automation.as_deref().unwrap_or(&[])
    }

    pub fn automations_mut(&mut self) -> &mut [Automation] {
        self.data.automation.as_deref_mut().unwrap_or(&mut [])
    }

    pub fn push_automation(&mut self, automation: Automation) {
        self.data
            .automation
            .get_or_insert(Vec::new())
            .push(automation);
    }

    pub fn textures(&self) -> &[Texture] {
        &self.textures
    }

    pub fn push_texture(&mut self, tex: Texture) -> u32 {
        let id = self.textures.len().try_into().unwrap();
        self.textures.push(tex);
        id
    }

    pub fn vendor_data(&self) -> &[VendorData] {
        &self.vendor_data
    }

    pub fn vendor_data_mut(&mut self) -> &mut [VendorData] {
        &mut self.vendor_data
    }

    pub fn push_vendor_data(&mut self, data: VendorData) {
        self.vendor_data.push(data);
    }
}

/// A texture image.
pub struct Texture {
    enc: TextureEncoding,
    data: Vec<u8>,
}

impl Texture {
    pub fn new(encoding: TextureEncoding, data: Vec<u8>) -> Self {
        Self {
            enc: encoding,
            data,
        }
    }

    pub fn encoding(&self) -> TextureEncoding {
        self.enc
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl fmt::Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Texture")
            .field("encoding", &self.enc)
            .field("data_len", &self.data.len())
            .finish()
    }
}

/// List of supported formats for [`Texture`]s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TextureEncoding {
    /// Texture is PNG encoded (lossless).
    Png = 0,
    /// Texture is in Truevision TGA / TARGA format (lossless).
    Tga = 1,
    /// [Not yet implemented] Texture is BC7 compressed (lossy).
    Bc7 = 2,
}

/// Vendor-specific extension data attached to a model.
pub struct VendorData {
    name: String,
    payload: Vec<u8>,
}

impl VendorData {
    pub fn new(name: String, data: Vec<u8>) -> Self {
        Self {
            name,
            payload: data,
        }
    }

    /// The name identifying the application this data is intended for.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the payload data.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

impl fmt::Debug for VendorData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VendorData")
            .field("name", &self.name)
            .field("payload", &self.payload.escape_ascii())
            .finish()
    }
}

/// Root JSON object.
#[derive(Debug, Serialize, Deserialize)]
struct JsonData {
    meta: Metadata,
    physics: Physics,
    nodes: Node, // really the root node
    param: Vec<Param>,
    automation: Option<Vec<Automation>>,
}

/// A unique ID attached to some model entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Uuid {
    raw: u64,
}

impl Uuid {
    pub fn raw(&self) -> u64 {
        self.raw
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(f)
    }
}

/// A vector or point in 2D space.
pub type Vec2 = [f32; 2];

/// A vector or point in 3D space.
pub type Vec3 = [f32; 3];
