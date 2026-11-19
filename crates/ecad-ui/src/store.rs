//! Datos del proyecto en memoria del servidor (solo prueba de concepto: a
//! partir del sprint 3 salen de la base de datos de revisiones).

use std::collections::HashMap;
use std::sync::Arc;

use crate::{PageInfo, ProjectInfo};

#[derive(Debug, Clone)]
pub struct StoredPage {
    pub title: String,
    /// SVG ya saneado, listo para incrustar.
    pub svg: String,
}

#[derive(Debug, Default)]
pub struct Project {
    pub name: String,
    pub pages: Vec<StoredPage>,
    /// Imágenes que enlazan las páginas (`EPLAN.png`), por nombre de fichero.
    pub assets: HashMap<String, Vec<u8>>,
}

/// Proyecto compartido entre las peticiones (contexto de Leptos y extensión
/// de Axum).
#[derive(Debug, Clone, Default)]
pub struct ProjectStore(pub Arc<Project>);

impl ProjectStore {
    pub fn info(&self) -> ProjectInfo {
        ProjectInfo {
            name: self.0.name.clone(),
            pages: self
                .0
                .pages
                .iter()
                .enumerate()
                .map(|(i, p)| PageInfo {
                    number: i + 1,
                    title: p.title.clone(),
                })
                .collect(),
        }
    }

    /// Página por número (desde 1).
    pub fn page(&self, number: usize) -> Option<&StoredPage> {
        self.0.pages.get(number.checked_sub(1)?)
    }
}
