//! Lectura de los formatos que exporta EPLAN.
//!
//! - [`archive`]: el contenedor `.epdz` (7z) cargado en memoria.
//! - [`epdz`]: el archivo clasificado (páginas, E3D, AML) con `manifest.db`.
//! - [`manifest`]: `manifest.db` leído a estructuras tipadas.
//! - [`project`]: el modelo de proyecto (cabecera, estructura, páginas,
//!   funciones, conexiones…).
//! - [`designation`]: designaciones EPLAN (`=VA2+A1-FC1:6/T3`).
//! - [`svg`]: páginas de esquema SVG, saneadas para incrustarlas en HTML.
//!
//! Sin red ni interfaz: este crate solo lee y transforma datos.

pub mod archive;
pub mod designation;
pub mod epdz;
pub mod manifest;
pub mod project;
pub mod svg;

pub use archive::{Archive, Entry};
pub use designation::Designation;
pub use epdz::{EntryKind, Epdz};
pub use manifest::Manifest;
pub use project::Project;

/// Errores de lectura de `ecad-core`.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no se pudo leer el archivo 7z: {0}")]
    SevenZ(#[from] sevenz_rust2::Error),
    #[error("error de E/S: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQLite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("XML no válido: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("atributo XML no válido: {0}")]
    XmlAttr(#[from] quick_xml::events::attributes::AttrError),
    #[error("falta {0} en el archivo")]
    Missing(&'static str),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
