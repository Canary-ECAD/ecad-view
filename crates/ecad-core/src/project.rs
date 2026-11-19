//! Modelo de proyecto construido a partir de `manifest.db`.
//!
//! Qué significa cada propiedad y con qué seguridad está en
//! `docs/v2/modelo-datos.md`. Las que no están confirmadas contra EPLAN van
//! envueltas en [`Unverified`] (con su estado) o se guardan en bruto en
//! `unidentified`, para que nadie las muestre como dato seguro.

use std::collections::{BTreeMap, HashMap};

use serde::Serialize;

use crate::designation::{Designation, split_exit_direction};
use crate::manifest::{Manifest, Package};

/// Seguridad de un dato que todavía no se ha validado contra EPLAN.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Certainty {
    /// El significado encaja con todos los datos del ejemplo, pero falta
    /// compararlo con EPLAN.
    Probable,
    /// Hay una hipótesis, pero no se ha comprobado.
    Unconfirmed,
}

/// Dato con su grado de seguridad. Se serializa como
/// `{"value": …, "certainty": "probable"}`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Unverified<T> {
    pub value: T,
    pub certainty: Certainty,
}

impl<T> Unverified<T> {
    fn probable(value: T) -> Self {
        Self {
            value,
            certainty: Certainty::Probable,
        }
    }

    fn unconfirmed(value: T) -> Self {
        Self {
            value,
            certainty: Certainty::Unconfirmed,
        }
    }
}

/// Proyecto completo.
#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub header: Header,
    pub structure: Vec<StructureNode>,
    pub pages: Vec<Page>,
    pub installation_spaces: Vec<InstallationSpace>,
    pub functions: Vec<Function>,
    pub connections: Vec<Connection>,
    pub interruption_points: Vec<InterruptionPoint>,
    /// Número de objetos por tipo en `manifest.db`.
    pub counts: BTreeMap<String, usize>,
}

/// Cabecera del proyecto (objeto `project`).
#[derive(Debug, Clone, Default, Serialize)]
pub struct Header {
    /// ep.10000
    pub name: Option<String>,
    /// ep.10011
    pub description: Option<String>,
    /// ep.10013 (número de proyecto / pedido)
    pub number: Option<String>,
    /// ep.10014 · ep.10015 · ep.10016 / 10017
    pub customer: Customer,
    /// ep.10100: empresa que hace el proyecto.
    pub company: Option<String>,
    /// ep.10042
    pub year: Option<String>,
    /// `language`, p. ej. `es_ES`.
    pub language: Option<String>,
    /// ep.10023: identificador interno del proyecto en EPLAN.
    pub project_id: Option<String>,
    /// ep.3010: equipo desde el que se exportó.
    pub exported_from: Option<String>,
    /// ep.3011: usuario que exportó.
    pub exported_by: Option<String>,
    /// Versión de EPLAN que exportó (tabla `manifest`).
    pub eplan_version: Option<String>,
    /// Versión del formato del manifest (tabla `manifest`).
    pub manifest_version: Option<String>,
    /// Todas las propiedades del objeto `project`, en bruto.
    pub properties: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Customer {
    pub short_name: Option<String>,
    pub name: Option<String>,
    pub address: Vec<String>,
}

/// Aspecto de la estructura del proyecto (objetos `location`).
#[derive(Debug, Clone, Serialize)]
pub struct StructureNode {
    pub id: i64,
    pub kind: StructureKind,
    /// Sin prefijo (`B2.X1`). Vacío para el nodo raíz de cada aspecto.
    pub code: String,
    /// Con prefijo (`+B2.X1`).
    pub full: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StructureKind {
    /// `=` designación de función (typenamenr 10002).
    Function,
    /// `+` designación de emplazamiento (10004).
    Location,
    /// `&` tipo de documento (10006).
    DocumentType,
    Other,
}

/// Página de esquema.
#[derive(Debug, Clone, Serialize)]
pub struct Page {
    pub id: i64,
    /// Nombre del objeto (`Sample_Project-4_21`).
    pub name: String,
    /// Ruta del SVG dentro del `.epdz`.
    pub file: Option<String>,
    /// ep.11009: `=GB1+A1&EFS1/1`.
    pub designation: Designation,
    /// ep.11000
    pub counter: Option<String>,
    /// ep.11011
    pub description: Option<String>,
    /// ep.11017: tipo de página (`Esquema multipolar`).
    pub page_type: Option<String>,
    /// Funciones que aparecen en la página (`page_functions`).
    pub functions: Vec<i64>,
    /// Puntos de interrupción de la página (`page_interruptionpoints`).
    pub interruption_points: Vec<i64>,
}

/// Espacio de instalación (armario, placa) con su modelo E3D.
#[derive(Debug, Clone, Serialize)]
pub struct InstallationSpace {
    pub id: i64,
    pub name: String,
    /// Ruta del `.E3d` dentro del `.epdz`.
    pub file: Option<String>,
    /// ep.36018: `Armario 1`.
    pub description: Option<String>,
    /// ep.36452: `A1`.
    pub code: Option<String>,
    /// ep.36454: `+A1 (A1)`.
    pub label: Option<String>,
    /// Identificador EPLAN (ep.2000, `253/183/38195/0`). Las conexiones
    /// enrutadas lo referencian.
    pub oid: Option<String>,
}

/// Función EPLAN: dispositivo, borne, punto de conexión…
#[derive(Debug, Clone, Serialize)]
pub struct Function {
    pub id: i64,
    pub name: String,
    /// Id del elemento en el SVG de la página (`Id17_4180`).
    pub svg_id: Option<String>,
    /// Identificador EPLAN (ep.2000).
    pub oid: Option<String>,
    /// ep.20001
    pub designation: Designation,
    /// ep.20006: dispositivo al que pertenece (`+A1-XD5`).
    pub device: Option<String>,
    pub kind: FunctionKind,
    /// ep.20188: definición de función EPLAN (`100/6/1`).
    pub category: Option<String>,
    /// ep.20026: nombre de la definición (`Borne de paso`), en el idioma
    /// del proyecto.
    pub definition: Option<String>,
    /// ep.20031: texto de función.
    pub text: Option<String>,
    /// ep.20027: datos técnicos (`0,55 kW`).
    pub technical: Option<String>,
    /// ep.20030: número de borne / punto.
    pub terminal_number: Option<String>,
    /// ep.20038: designaciones de los puntos de conexión (`x1;x2`).
    pub connection_points: Vec<String>,
    /// ep.1250: descripción del emplazamiento.
    pub location_description: Option<String>,
    /// ep.19007: referencia cruzada (`=VA1+A1&EFS1/ 1.7`).
    pub cross_reference: Option<String>,
    /// Piezas (ep.20100 · ep.20193 · ep.20919 por `propindex`).
    pub parts: Vec<Part>,
    /// Páginas en las que aparece.
    pub pages: Vec<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionKind {
    Device,
    Terminal,
    TerminalStrip,
    ConnectionPoint,
    Cable,
    /// Colocación de artículo en el montaje (categoría 8).
    Placement,
    /// Componente mecánico de armario: perfiles, canaletas, carriles
    /// (categorías 15000–15999).
    Mounting,
    /// Ruta de topología (categoría 21).
    Topology,
}

impl FunctionKind {
    /// Clasificación por la categoría de la definición de función (primer
    /// número de ep.20188). Deducida de los nombres que da EPLAN (ep.20026).
    fn from_category(category: Option<&str>) -> Self {
        let first = category
            .and_then(|c| c.split('/').next())
            .and_then(|c| c.parse::<u32>().ok());
        match first {
            Some(100 | 101) => Self::Terminal,
            Some(102) => Self::TerminalStrip,
            Some(300 | 1302 | 1304 | 6002) => Self::ConnectionPoint,
            Some(4 | 400 | 401) => Self::Cable,
            Some(8) => Self::Placement,
            Some(15000..=15999) => Self::Mounting,
            Some(21) => Self::Topology,
            _ => Self::Device,
        }
    }
}

/// Pieza (artículo) de una función.
#[derive(Debug, Clone, Serialize)]
pub struct Part {
    /// ep.20100: `PXC.3211813`.
    pub number: String,
    /// ep.20193
    pub description: Option<String>,
    /// ep.20919: número de pedido del fabricante.
    pub order_number: Option<String>,
}

/// Conexión (objeto `mergedconnection`): un cable, puente o tubo que se
/// fabrica, con sus dos extremos.
#[derive(Debug, Clone, Serialize)]
pub struct Connection {
    pub id: i64,
    pub name: String,
    pub svg_id: Option<String>,
    pub kind: ConnectionKind,
    pub source: ConnectionEnd,
    pub target: ConnectionEnd,
    /// ep.31004
    pub color: Option<String>,
    /// ep.31007: sección con unidad (`0,25 mm²`) o diámetro de tubo (`8mm`).
    pub cross_section: Option<String>,
    /// ep.31002: solo el valor.
    pub cross_section_value: Option<String>,
    /// ep.31060: solo la unidad.
    pub cross_section_unit: Option<String>,
    /// ep.31003: longitud con unidad (`0,18 m`).
    pub length: Option<String>,
    /// ep.31000: longitud en metros.
    pub length_m: Option<f64>,
    /// ep.31048: tipo de conductor o tubo.
    pub conductor_type: Option<String>,
    /// ep.33003
    pub potential: Option<String>,
    /// ep.20045
    pub signal: Option<String>,
    /// ep.20006: cable multipolar al que pertenece el conductor (`+A2-WD8`).
    pub cable: Option<String>,
    /// Cadena de puentes (`daisychainid` · `daisyChainorderid`). `None` si
    /// la conexión no forma parte de una cadena.
    pub daisy_chain: Option<DaisyChain>,
    /// ep.20188: definición de función de la conexión (`4/1/4`).
    pub category: Option<String>,
    /// `connection3did`: pieza 3D del cable enrutado (`166/60765`).
    pub wire_3d: Option<String>,
    /// `connection3dinstallationspaceoid`: espacio de instalación del
    /// enrutado.
    pub installation_space: Option<String>,
    /// `connectionoid`: conexiones EPLAN originales agrupadas.
    pub connection_oids: Vec<String>,
    /// ep.31095: canaletas o tramos por los que pasa, en orden.
    pub route: Option<Unverified<Vec<String>>>,
    /// ep.31011: designación de la conexión (número de hilo o de tubo).
    pub designation: Option<Unverified<String>>,
    /// Propiedades sin identificar (ep.31006, 31058, 31092, 31114, 31115,
    /// 31155…), en bruto.
    pub unidentified: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionKind {
    /// Hilo suelto entre dos puntos.
    Wire,
    /// Puente entre puntos del mismo dispositivo (regleta, bornes).
    Bridge,
    /// Tubo neumático.
    Tube,
    /// Conductor de un cable multipolar.
    CableCore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DaisyChain {
    pub id: i64,
    pub order: i64,
}

/// Un extremo de una conexión.
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionEnd {
    /// ep.31019 (origen) / ep.31020 (destino).
    pub designation: Designation,
    /// ep.31051 / ep.31052: `End sleeve` (puntera), `Stripping` (pelado).
    pub treatment: Option<String>,
    /// ep.31055 / ep.31056: longitud de pelado (`10 mm`).
    pub strip_length: Option<Unverified<String>>,
    /// Sufijo de ep.31144 / ep.31145 (`HL`, `TR`, `R`…).
    pub exit_direction: Option<Unverified<String>>,
    /// ep.31148–31150 (origen) / ep.31151–31153 (destino): x, y, z en mm.
    pub position: Option<Unverified<[f64; 3]>>,
    /// `source.35575` / `destination.35575`: ¿par de apriete?
    pub torque: Option<Unverified<String>>,
    /// `source.35576` / `destination.35576`: ¿tamaño de herramienta?
    pub tool_size: Option<Unverified<String>>,
}

/// Punto de interrupción (salto de potencial entre páginas).
#[derive(Debug, Clone, Serialize)]
pub struct InterruptionPoint {
    pub id: i64,
    pub name: String,
    pub svg_id: Option<String>,
    /// ep.20001: `=GB1-24V`.
    pub designation: Designation,
    /// ep.19007: referencia cruzada resuelta por EPLAN.
    pub cross_reference: Option<String>,
    pub pages: Vec<i64>,
}

/// Propiedades de conexión cuyo significado no se conoce todavía.
const UNIDENTIFIED: &[&str] = &[
    "ep.31006",
    "ep.31058",
    "ep.31092",
    "ep.31114",
    "ep.31115",
    "ep.31155",
    "source.35573",
    "destination.35573",
    "source.35578",
    "destination.35578",
];

impl Project {
    pub fn from_manifest(m: &Manifest) -> Self {
        let mut counts = BTreeMap::new();
        for p in &m.packages {
            *counts.entry(p.kind.clone()).or_insert(0) += 1;
        }

        let mut function_pages: HashMap<i64, Vec<i64>> = HashMap::new();
        let mut page_functions: HashMap<i64, Vec<i64>> = HashMap::new();
        for &(page, function) in &m.page_functions {
            function_pages.entry(function).or_default().push(page);
            page_functions.entry(page).or_default().push(function);
        }
        let mut point_pages: HashMap<i64, Vec<i64>> = HashMap::new();
        let mut page_points: HashMap<i64, Vec<i64>> = HashMap::new();
        for &(page, point) in &m.page_interruption_points {
            point_pages.entry(point).or_default().push(page);
            page_points.entry(page).or_default().push(point);
        }

        let header = m
            .of_kind("project")
            .next()
            .map(|p| header(p, m))
            .unwrap_or_default();

        let structure = m.of_kind("location").map(structure_node).collect();

        let pages = m
            .of_kind("page")
            .map(|p| Page {
                id: p.id,
                name: p.name.clone(),
                file: file_with_extension(m, p, "svg"),
                designation: Designation::parse(p.prop(11009).unwrap_or_default()),
                counter: owned(p.prop(11000)),
                description: owned(p.prop(11011)),
                page_type: owned(p.prop(11017)),
                functions: page_functions.remove(&p.id).unwrap_or_default(),
                interruption_points: page_points.remove(&p.id).unwrap_or_default(),
            })
            .collect();

        let installation_spaces = m
            .of_kind("installationspace")
            .map(|p| InstallationSpace {
                id: p.id,
                name: p.name.clone(),
                file: file_with_extension(m, p, "e3d"),
                description: owned(p.prop(36018)),
                code: owned(p.prop(36452)),
                label: owned(p.prop(36454)),
                oid: owned(p.prop(2000)),
            })
            .collect();

        let functions = m
            .of_kind("function")
            .map(|p| function(p, function_pages.remove(&p.id).unwrap_or_default()))
            .collect();

        let connections = m.of_kind("mergedconnection").map(connection).collect();

        let interruption_points = m
            .of_kind("interruptionpoint")
            .map(|p| InterruptionPoint {
                id: p.id,
                name: p.name.clone(),
                svg_id: svg_id(&p.name),
                designation: Designation::parse(p.prop(20001).unwrap_or_default()),
                cross_reference: owned(p.prop(19007)),
                pages: point_pages.remove(&p.id).unwrap_or_default(),
            })
            .collect();

        Self {
            header,
            structure,
            pages,
            installation_spaces,
            functions,
            connections,
            interruption_points,
            counts,
        }
    }

    /// Página por id de objeto.
    pub fn page(&self, id: i64) -> Option<&Page> {
        self.pages.iter().find(|p| p.id == id)
    }
}

fn header(p: &Package, m: &Manifest) -> Header {
    Header {
        name: owned(p.prop(10000)),
        description: owned(p.prop(10011)),
        number: owned(p.prop(10013)),
        customer: Customer {
            short_name: owned(p.prop(10014)),
            name: owned(p.prop(10015)),
            address: [10016, 10017]
                .iter()
                .filter_map(|&id| owned(p.prop(id)))
                .collect(),
        },
        company: owned(p.prop(10100)),
        year: owned(p.prop(10042)),
        language: owned(p.named("language")),
        project_id: owned(p.prop(10023)),
        exported_from: owned(p.prop(3010)),
        exported_by: owned(p.prop(3011)),
        eplan_version: m.eplan_version.clone(),
        manifest_version: m.manifest_version.clone(),
        properties: p
            .properties
            .iter()
            .filter(|x| !x.value.is_empty())
            .map(|x| (x.key(), x.value.clone()))
            .collect(),
    }
}

fn structure_node(p: &Package) -> StructureNode {
    let kind = match p.named("typenamenr") {
        Some("10002") => StructureKind::Function,
        Some("10004") => StructureKind::Location,
        Some("10006") => StructureKind::DocumentType,
        _ => StructureKind::Other,
    };
    StructureNode {
        id: p.id,
        kind,
        code: p.named("fullname").unwrap_or_default().to_owned(),
        full: p.named("fullnamewithprefix").unwrap_or_default().to_owned(),
        description: owned(p.named("description")),
    }
}

fn function(p: &Package, pages: Vec<i64>) -> Function {
    let category = owned(p.prop(20188));
    // Las piezas van indexadas por propindex: ep.20100, 20193 y 20919 con el
    // mismo índice describen la misma pieza.
    let mut parts: BTreeMap<i64, Part> = BTreeMap::new();
    for prop in p.properties.iter().filter(|x| !x.value.is_empty()) {
        let index = prop.index.unwrap_or(0);
        match prop.id {
            Some(20100) => {
                parts
                    .entry(index)
                    .or_insert_with(|| Part {
                        number: String::new(),
                        description: None,
                        order_number: None,
                    })
                    .number = prop.value.clone();
            }
            Some(20193) => {
                if let Some(part) = parts.get_mut(&index) {
                    part.description = Some(prop.value.clone());
                }
            }
            Some(20919) => {
                if let Some(part) = parts.get_mut(&index) {
                    part.order_number = Some(prop.value.clone());
                }
            }
            _ => {}
        }
    }
    Function {
        id: p.id,
        name: p.name.clone(),
        svg_id: svg_id(&p.name),
        oid: owned(p.prop(2000)),
        designation: Designation::parse(p.prop(20001).unwrap_or_default()),
        device: owned(p.prop(20006)),
        kind: FunctionKind::from_category(category.as_deref()),
        category,
        definition: owned(p.prop(20026)),
        text: owned(p.prop(20031)),
        technical: owned(p.prop(20027)),
        terminal_number: owned(p.prop(20030)),
        connection_points: p.prop(20038).map(|v| list(v, ';')).unwrap_or_default(),
        location_description: owned(p.prop(1250)),
        cross_reference: owned(p.prop(19007)),
        parts: parts
            .into_values()
            .filter(|x| !x.number.is_empty())
            .collect(),
        pages,
    }
}

fn connection(p: &Package) -> Connection {
    let source = Designation::parse(p.prop(31019).unwrap_or_default());
    let target = Designation::parse(p.prop(31020).unwrap_or_default());

    // ep.31144/31145 repiten origen y destino con la dirección de salida,
    // pero no siempre en ese orden (en el ejemplo, 28 de 545 van cruzados):
    // se asignan por la designación.
    let mut source_dir = None;
    let mut target_dir = None;
    for value in [p.prop(31144), p.prop(31145)].into_iter().flatten() {
        let (designation, dir) = split_exit_direction(value);
        let Some(dir) = dir else { continue };
        let dir = Some(Unverified::unconfirmed(dir.to_owned()));
        if designation == source.full && source_dir.is_none() {
            source_dir = dir;
        } else if designation == target.full && target_dir.is_none() {
            target_dir = dir;
        }
    }

    let cable = p
        .prop(20006)
        .filter(|c| Designation::parse(c).device.is_some())
        .map(str::to_owned);
    let category = owned(p.prop(20188));
    let cross_section = owned(p.prop(31007));
    let cross_section_unit = owned(p.prop(31060));
    let conductor_type = owned(p.prop(31048));

    let kind = if category.as_deref() == Some("4/1/15")
        || conductor_type
            .as_deref()
            .is_some_and(|t| t.starts_with("PUN"))
        || cross_section_unit.is_none() && cross_section.as_deref().is_some_and(is_diameter)
    {
        ConnectionKind::Tube
    } else if cable.is_some() {
        ConnectionKind::CableCore
    } else if source.same_device(&target) {
        ConnectionKind::Bridge
    } else {
        ConnectionKind::Wire
    };

    let daisy_chain = match (
        p.named("daisychainid").and_then(|v| v.parse().ok()),
        p.named("daisyChainorderid").and_then(|v| v.parse().ok()),
    ) {
        (Some(id), order) if id != 0 => Some(DaisyChain {
            id,
            order: order.unwrap_or(0),
        }),
        _ => None,
    };

    let unidentified = p
        .properties
        .iter()
        .filter(|x| !x.value.is_empty())
        .map(|x| (x.key(), x.value.clone()))
        .filter(|(k, _)| UNIDENTIFIED.contains(&k.as_str()))
        .collect();

    Connection {
        id: p.id,
        name: p.name.clone(),
        svg_id: svg_id(&p.name),
        kind,
        source: end(p, source, "source", [31051, 31055, 31148], source_dir),
        target: end(p, target, "destination", [31052, 31056, 31151], target_dir),
        color: owned(p.prop(31004)),
        cross_section,
        cross_section_value: owned(p.prop(31002)),
        cross_section_unit,
        length: owned(p.prop(31003)),
        length_m: p.prop(31000).and_then(|v| v.parse().ok()),
        conductor_type,
        potential: owned(p.prop(33003)),
        signal: owned(p.prop(20045)),
        cable,
        daisy_chain,
        category,
        wire_3d: owned(p.named("connection3did")),
        installation_space: owned(p.named("connection3dinstallationspaceoid")),
        connection_oids: p.named_all("connectionoid").map(str::to_owned).collect(),
        route: p.prop(31095).map(|v| Unverified::probable(list(v, ';'))),
        designation: p.prop(31011).map(|v| Unverified::probable(v.to_owned())),
        unidentified,
    }
}

/// Un extremo: `ids` son la propiedad de tratamiento, la de pelado y la
/// primera de las tres coordenadas.
fn end(
    p: &Package,
    designation: Designation,
    prefix: &str,
    [treatment, strip, x]: [i64; 3],
    exit_direction: Option<Unverified<String>>,
) -> ConnectionEnd {
    let coord = |id: i64| p.prop(id).and_then(|v| v.parse::<f64>().ok());
    let position = match (coord(x), coord(x + 1), coord(x + 2)) {
        (Some(x), Some(y), Some(z)) => Some(Unverified::probable([x, y, z])),
        _ => None,
    };
    // `0` en estas dos es "sin dato".
    let named = |id: &str| {
        p.named(&format!("{prefix}.{id}"))
            .filter(|v| *v != "0")
            .map(|v| Unverified::unconfirmed(v.to_owned()))
    };
    ConnectionEnd {
        designation,
        treatment: owned(p.prop(treatment)),
        strip_length: p.prop(strip).map(|v| Unverified::probable(v.to_owned())),
        exit_direction,
        position,
        torque: named("35575"),
        tool_size: named("35576"),
    }
}

/// Fichero propio del objeto con la extensión dada, con su ruta completa en
/// el `.epdz` (`packages/<contenedor>/items/<tipo>/<locator>`).
fn file_with_extension(m: &Manifest, p: &Package, ext: &str) -> Option<String> {
    let item = m.files(p.id).find(|i| {
        i.locator
            .rsplit_once('.')
            .is_some_and(|(_, e)| e.eq_ignore_ascii_case(ext))
    })?;
    let container = p.parent.and_then(|id| m.package(id)).map(|c| &c.name)?;
    Some(format!(
        "packages/{container}/items/{}/{}",
        item.kind, item.locator
    ))
}

/// `_17_4180` al final del nombre → `Id17_4180` (id del elemento en el SVG).
fn svg_id(name: &str) -> Option<String> {
    let mut parts = name.rsplitn(3, '_');
    let object = parts.next()?;
    let kind = parts.next()?;
    let is_num = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    (is_num(kind) && is_num(object)).then(|| format!("Id{kind}_{object}"))
}

/// Diámetro de tubo: un número seguido de `mm` sin `²`.
fn is_diameter(text: &str) -> bool {
    text.strip_suffix("mm").is_some_and(|n| {
        !n.is_empty()
            && n.trim()
                .chars()
                .all(|c| c.is_ascii_digit() || c == ',' || c == '.')
    })
}

fn list(value: &str, sep: char) -> Vec<String> {
    value
        .split(sep)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

fn owned(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_ids() {
        assert_eq!(svg_id("_10007_80833").as_deref(), Some("Id10007_80833"));
        assert_eq!(
            svg_id("=VA2-TA1-XD12.1_70_1887").as_deref(),
            Some("Id70_1887")
        );
        assert_eq!(svg_id("Sample_Project-4_21"), None);
        assert_eq!(svg_id("sin_id"), None);
    }

    #[test]
    fn diameters() {
        assert!(is_diameter("8mm"));
        assert!(is_diameter("12mm"));
        assert!(!is_diameter("0,25 mm²"));
        assert!(!is_diameter("mm"));
    }

    #[test]
    fn function_kinds() {
        assert_eq!(
            FunctionKind::from_category(Some("100/6/1")),
            FunctionKind::Terminal
        );
        assert_eq!(
            FunctionKind::from_category(Some("15104/1/1")),
            FunctionKind::Mounting
        );
        assert_eq!(
            FunctionKind::from_category(Some("501/1/1")),
            FunctionKind::Device
        );
        assert_eq!(FunctionKind::from_category(None), FunctionKind::Device);
    }
}
