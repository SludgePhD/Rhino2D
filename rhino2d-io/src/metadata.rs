use serde::{Deserialize, Serialize};

/// Model metadata containing name and author information.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    name: Option<String>,
    /// Version of the program that produced this file (eg. `v0.7.1-13-ge8c15a3`).
    version: String,
    rigger: Option<String>,
    artist: Option<String>,
    rights: Option<String>,
    copyright: Option<String>,
    #[serde(rename = "licenseURL")]
    license_url: Option<String>,
    contact: Option<String>,
    reference: Option<String>,
    // FIXME apparently this is `u32::MAX` when unset?
    thumbnail_id: Option<u32>,
    preserve_pixels: bool,
}

impl Metadata {
    pub fn new(version: String) -> Self {
        Self {
            version,
            name: None,
            rigger: None,
            artist: None,
            rights: None,
            copyright: None,
            license_url: None,
            contact: None,
            reference: None,
            thumbnail_id: None,
            preserve_pixels: false,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }

    /// Returns the version of the program that created this file (*not* the version of the model).
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Sets the version of the program that created this file (*not* the version of the model).
    pub fn set_version(&mut self, version: String) {
        self.version = version;
    }

    pub fn rigger(&self) -> Option<&str> {
        self.rigger.as_deref()
    }

    pub fn set_rigger(&mut self, rigger: Option<String>) {
        self.rigger = rigger;
    }

    pub fn artist(&self) -> Option<&str> {
        self.artist.as_deref()
    }

    pub fn set_artist(&mut self, artist: Option<String>) {
        self.artist = artist;
    }

    pub fn rights(&self) -> Option<&str> {
        self.rights.as_deref()
    }

    pub fn set_rights(&mut self, rights: Option<String>) {
        self.rights = rights;
    }

    pub fn copyright(&self) -> Option<&str> {
        self.copyright.as_deref()
    }

    pub fn set_copyright(&mut self, copyright: Option<String>) {
        self.copyright = copyright;
    }

    pub fn license_url(&self) -> Option<&str> {
        self.license_url.as_deref()
    }

    pub fn set_license_url(&mut self, license_url: Option<String>) {
        self.license_url = license_url;
    }

    pub fn contact(&self) -> Option<&str> {
        self.contact.as_deref()
    }

    pub fn set_contact(&mut self, contact: Option<String>) {
        self.contact = contact;
    }

    pub fn reference(&self) -> Option<&str> {
        self.reference.as_deref()
    }

    pub fn set_reference(&mut self, reference: Option<String>) {
        self.reference = reference;
    }

    /// Returns the Texture ID to use as a model thumbnail/preview.
    pub fn thumbnail_id(&self) -> Option<u32> {
        if self.thumbnail_id == Some(u32::MAX) {
            None
        } else {
            self.thumbnail_id
        }
    }

    pub fn set_thumbnail_id(&mut self, thumbnail_id: Option<u32>) {
        self.thumbnail_id = thumbnail_id;
    }

    pub fn preserve_pixels(&self) -> bool {
        self.preserve_pixels
    }

    pub fn set_preserve_pixels(&mut self, preserve_pixels: bool) {
        self.preserve_pixels = preserve_pixels;
    }
}
