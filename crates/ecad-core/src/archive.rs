//! Contenedor `.epdz`: un archivo 7z que se descomprime entero en memoria.
//!
//! EPLAN genera 7z sólidos, así que leer una entrada suelta obliga a
//! descomprimir las anteriores del mismo bloque. Con proyectos de pocos MB es
//! más simple y rápido extraerlo todo una vez.

use std::io::{Read, Seek};
use std::path::Path;

use sevenz_rust2::{ArchiveReader, Password};

use crate::Result;

/// Un fichero del archivo, con la ruta normalizada a `/`.
#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub data: Vec<u8>,
}

/// Archivo 7z cargado en memoria.
#[derive(Debug, Clone, Default)]
pub struct Archive {
    entries: Vec<Entry>,
}

impl Archive {
    /// Abre y descomprime un `.epdz` desde disco.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let reader = ArchiveReader::open(path, Password::empty())?;
        Self::read_all(reader)
    }

    /// Descomprime un `.epdz` desde cualquier lector (por ejemplo, una subida).
    pub fn from_reader<R: Read + Seek>(source: R) -> Result<Self> {
        let reader = ArchiveReader::new(source, Password::empty())?;
        Self::read_all(reader)
    }

    fn read_all<R: Read + Seek>(mut reader: ArchiveReader<R>) -> Result<Self> {
        let mut entries = Vec::new();
        reader.for_each_entries(|entry, data| {
            if !entry.is_directory() {
                let mut buf = Vec::with_capacity(entry.size() as usize);
                data.read_to_end(&mut buf)?;
                entries.push(Entry {
                    name: entry.name().replace('\\', "/"),
                    data: buf,
                });
            }
            Ok(true)
        })?;
        Ok(Self { entries })
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Busca una entrada por ruta exacta (con `/`).
    pub fn get(&self, name: &str) -> Option<&[u8]> {
        self.entries
            .iter()
            .find(|e| e.name == name)
            .map(|e| e.data.as_slice())
    }

    /// Entradas con la extensión dada, sin distinguir mayúsculas.
    pub fn with_extension<'a>(&'a self, ext: &'a str) -> impl Iterator<Item = &'a Entry> + 'a {
        self.entries.iter().filter(move |e| {
            Path::new(&e.name)
                .extension()
                .is_some_and(|x| x.eq_ignore_ascii_case(ext))
        })
    }
}
