//! `manifest.db`: la base SQLite que EPLAN incluye en el `.epdz`.
//!
//! Se carga entera en memoria (`sqlite3_deserialize`, sin fichero temporal) y
//! se lee de una vez a estructuras tipadas: objetos (`package`), propiedades
//! agrupadas por objeto, ficheros (`item`) y las relaciones página → función y
//! página → punto de interrupción. El modelo de proyecto
//! ([`crate::project`]) se construye encima.

use std::collections::HashMap;

use rusqlite::{Connection, OpenFlags};

use crate::Result;

/// Una propiedad de un objeto. EPLAN las identifica por número (`propid`,
/// p. ej. 31019) o por nombre (`propname`, p. ej. `connectionoid`,
/// `source.35575`).
#[derive(Debug, Clone)]
pub struct Property {
    pub name: Option<String>,
    pub id: Option<i64>,
    /// Índice para propiedades repetidas (varias piezas en una función).
    pub index: Option<i64>,
    pub value: String,
}

impl Property {
    /// Clave legible: `ep.31019` o el nombre (`connectionoid`).
    pub fn key(&self) -> String {
        match (&self.name, self.id) {
            (Some(name), _) => name.clone(),
            (None, Some(id)) => format!("ep.{id}"),
            (None, None) => String::new(),
        }
    }
}

/// Un objeto de la tabla `package`.
#[derive(Debug, Clone)]
pub struct Package {
    pub id: i64,
    pub parent: Option<i64>,
    pub kind: String,
    pub name: String,
    pub properties: Vec<Property>,
}

impl Package {
    /// Primer valor no vacío de la propiedad numérica `id` (por `propindex`).
    pub fn prop(&self, id: i64) -> Option<&str> {
        self.props(id).next()
    }

    /// Valores no vacíos de la propiedad numérica `id`, por `propindex`.
    pub fn props(&self, id: i64) -> impl Iterator<Item = &str> {
        self.properties
            .iter()
            .filter(move |p| p.id == Some(id) && p.name.is_none())
            .map(|p| p.value.as_str())
            .filter(|v| !v.is_empty())
    }

    /// Primer valor no vacío de la propiedad con nombre `name`.
    pub fn named(&self, name: &str) -> Option<&str> {
        self.properties
            .iter()
            .filter(|p| p.name.as_deref() == Some(name))
            .map(|p| p.value.as_str())
            .find(|v| !v.is_empty())
    }

    /// Valores no vacíos de la propiedad con nombre `name`, por `propindex`.
    pub fn named_all<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        self.properties
            .iter()
            .filter(move |p| p.name.as_deref() == Some(name))
            .map(|p| p.value.as_str())
            .filter(|v| !v.is_empty())
    }
}

/// Fichero del archivo asociado a un objeto (tabla `item`).
#[derive(Debug, Clone)]
pub struct Item {
    pub package: i64,
    pub kind: String,
    pub locator: String,
    /// Recurso compartido (logo, script) enlazado desde otro item.
    pub attached: bool,
}

/// Contenido de `manifest.db`.
#[derive(Debug, Clone, Default)]
pub struct Manifest {
    /// Tabla `database`: versión del formato de la base.
    pub database_version: Option<String>,
    /// Tabla `manifest`: versión de EPLAN que exportó (`2025.0.3.23756`).
    pub eplan_version: Option<String>,
    /// Tabla `manifest`: versión del esquema del manifest (`2.0`).
    pub manifest_version: Option<String>,
    /// Objetos en el orden de la tabla (el del navegador de EPLAN).
    pub packages: Vec<Package>,
    pub items: Vec<Item>,
    /// `page_functions`: (página, función).
    pub page_functions: Vec<(i64, i64)>,
    /// `page_interruptionpoints`: (página, punto de interrupción).
    pub page_interruption_points: Vec<(i64, i64)>,
    by_id: HashMap<i64, usize>,
}

impl Manifest {
    /// Lee un `manifest.db` desde sus bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut db = Connection::open_with_flags(
            ":memory:",
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        db.deserialize_read_exact("main", data, data.len(), true)?;
        Self::read(&db)
    }

    /// Lee un `manifest.db` ya abierto.
    pub fn read(db: &Connection) -> Result<Self> {
        let database_version = db
            .query_row("SELECT version FROM database LIMIT 1", [], |r| r.get(0))
            .ok();
        let (eplan_version, manifest_version) = db
            .query_row(
                "SELECT schemaversion, version FROM manifest ORDER BY id LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap_or((None, None));

        let mut packages: Vec<Package> = db
            .prepare("SELECT id, parentid, type, name FROM package ORDER BY id")?
            .query_map([], |r| {
                Ok(Package {
                    id: r.get(0)?,
                    parent: r.get(1)?,
                    kind: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    name: r.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    properties: Vec::new(),
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        let by_id: HashMap<i64, usize> = packages
            .iter()
            .enumerate()
            .map(|(i, p)| (p.id, i))
            .collect();

        let mut stmt = db.prepare(
            "SELECT packageid, propname, propid, propindex, value FROM property
             ORDER BY packageid, propindex, id",
        )?;
        let mut rows = stmt.query([])?;
        while let Some(r) = rows.next()? {
            let package: i64 = r.get(0)?;
            let Some(&i) = by_id.get(&package) else {
                continue;
            };
            packages[i].properties.push(Property {
                name: r.get(1)?,
                id: r.get(2)?,
                index: r.get(3)?,
                value: r.get::<_, Option<String>>(4)?.unwrap_or_default(),
            });
        }

        let items = db
            .prepare("SELECT packageid, type, locator, attatched FROM item ORDER BY id")?
            .query_map([], |r| {
                Ok(Item {
                    package: r.get(0)?,
                    kind: r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    locator: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    attached: r.get::<_, Option<bool>>(3)?.unwrap_or(false),
                })
            })?
            .collect::<rusqlite::Result<_>>()?;

        let page_functions = pairs(db, "SELECT pageid, functionid FROM page_functions")?;
        let page_interruption_points = pairs(
            db,
            "SELECT pageid, interruptionpointid FROM page_interruptionpoints",
        )?;

        Ok(Self {
            database_version,
            eplan_version,
            manifest_version,
            packages,
            items,
            page_functions,
            page_interruption_points,
            by_id,
        })
    }

    pub fn package(&self, id: i64) -> Option<&Package> {
        self.by_id.get(&id).map(|&i| &self.packages[i])
    }

    /// Objetos de un tipo (`mergedconnection`, `function`…), en orden.
    pub fn of_kind<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a Package> + 'a {
        self.packages.iter().filter(move |p| p.kind == kind)
    }

    /// Ficheros propios (no adjuntos) de un objeto.
    pub fn files(&self, package: i64) -> impl Iterator<Item = &Item> {
        self.items
            .iter()
            .filter(move |i| i.package == package && !i.attached)
    }
}

/// Tabla de relación con dos enteros. Si la tabla no existe (exportaciones
/// antiguas), vacía.
fn pairs(db: &Connection, sql: &str) -> Result<Vec<(i64, i64)>> {
    let mut stmt = match db.prepare(sql) {
        Ok(stmt) => stmt,
        Err(_) => return Ok(Vec::new()),
    };
    let rows = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}
