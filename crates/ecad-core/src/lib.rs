//! Lectura de los formatos que exporta EPLAN.
//!
//! - [`archive`]: el contenedor `.epdz` (7z) cargado en memoria.
//! - [`svg`]: páginas de esquema SVG, saneadas para incrustarlas en HTML.
//!
//! Sin red ni interfaz: este crate solo lee y transforma datos.

pub mod archive;
pub mod svg;

pub use archive::{Archive, Entry};

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
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
