//! Un `.epdz` abierto y clasificado: páginas, E3D, AML, `manifest.db` y
//! recursos de las páginas.

use std::io::{Read, Seek};
use std::path::Path;

use crate::manifest::Manifest;
use crate::project::Project;
use crate::{Archive, Entry, Error, Result};

/// Qué es cada entrada del archivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Manifest,
    /// Página de esquema (`.svg`).
    Page,
    /// Modelo 3D de un espacio de instalación (`.E3d`).
    Model,
    /// AutomationML (`.aml`).
    Aml,
    /// Imagen enlazada desde las páginas (logos, avisos).
    Image,
    /// Resto (`SVGScripting.js`…).
    Other,
}

impl EntryKind {
    pub fn of(name: &str) -> Self {
        if name.eq_ignore_ascii_case("manifest.db") {
            return Self::Manifest;
        }
        let ext = Path::new(name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        match ext.as_str() {
            "svg" => Self::Page,
            "e3d" => Self::Model,
            "aml" => Self::Aml,
            "png" | "jpg" | "jpeg" | "gif" | "bmp" => Self::Image,
            _ => Self::Other,
        }
    }
}

/// `.epdz` en memoria, con `manifest.db` ya leído.
#[derive(Debug, Clone)]
pub struct Epdz {
    archive: Archive,
    manifest: Manifest,
}

impl Epdz {
    /// Abre y descomprime un `.epdz` desde disco.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_archive(Archive::open(path)?)
    }

    /// Descomprime un `.epdz` desde cualquier lector (por ejemplo, una subida).
    pub fn from_reader<R: Read + Seek>(source: R) -> Result<Self> {
        Self::from_archive(Archive::from_reader(source)?)
    }

    pub fn from_archive(archive: Archive) -> Result<Self> {
        let db = archive
            .entries()
            .iter()
            .find(|e| EntryKind::of(&e.name) == EntryKind::Manifest)
            .ok_or(Error::Missing("manifest.db"))?;
        let manifest = Manifest::from_bytes(&db.data)?;
        Ok(Self { archive, manifest })
    }

    pub fn archive(&self) -> &Archive {
        &self.archive
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Construye el modelo de proyecto.
    pub fn project(&self) -> Project {
        Project::from_manifest(&self.manifest)
    }

    /// Entradas de un tipo, en el orden del archivo.
    pub fn entries(&self, kind: EntryKind) -> impl Iterator<Item = &Entry> {
        self.archive
            .entries()
            .iter()
            .filter(move |e| EntryKind::of(&e.name) == kind)
    }

    pub fn pages(&self) -> impl Iterator<Item = &Entry> {
        self.entries(EntryKind::Page)
    }

    pub fn models(&self) -> impl Iterator<Item = &Entry> {
        self.entries(EntryKind::Model)
    }

    pub fn amls(&self) -> impl Iterator<Item = &Entry> {
        self.entries(EntryKind::Aml)
    }

    pub fn images(&self) -> impl Iterator<Item = &Entry> {
        self.entries(EntryKind::Image)
    }

    /// Entrada por ruta exacta (las rutas de [`Project`] sirven tal cual).
    pub fn get(&self, name: &str) -> Option<&[u8]> {
        self.archive.get(name)
    }
}
