//! AutomationML (CAEX 2.15) que EPLAN exporta en `packages/automationML/`.
//!
//! El fichero del ejemplo ocupa 31 MB: se recorre una sola vez con
//! `quick-xml`, sin árbol en memoria, y solo se guarda lo que se usa.
//!
//! - `InstanceHierarchy`: el proyecto como árbol de [`Element`]: espacios de
//!   montaje → armario → placas y superficies de montaje → piezas, con su
//!   posición (`Frame`, relativa al padre), rol ProPanel, clase eCl@ss,
//!   artículo, taladros, puntos de fijación y conectores.
//! - `SystemUnitClassLib`: los artículos ([`Article`]): fabricante, GTIN,
//!   peso, enlaces a la ficha, dimensiones y fijación.
//!
//! Cada elemento lleva `Object identification` (`253/180/50377/0`), el mismo
//! valor que ep.2000 en el manifest y que `typeId/objectId` en el E3D. Qué se
//! sabe de cada atributo está en `docs/v2/modelo-datos.md`.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use quick_xml::XmlVersion;
use quick_xml::events::{BytesStart, Event};
use serde::Serialize;

use crate::project::Project;
use crate::{Error, Result};

/// Texto multilingüe: idioma (`es-ES`) → texto. La clave `""` es el valor
/// base del atributo (el idioma del proyecto).
pub type MultiText = BTreeMap<String, String>;

/// AutomationML leído.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Aml {
    pub writer: Writer,
    /// Nombre del proyecto (elemento raíz).
    pub project_name: Option<String>,
    pub project_description: Option<MultiText>,
    /// Idiomas presentes en los textos, ordenados.
    pub languages: Vec<String>,
    pub elements: Vec<Element>,
    /// Artículos por clave (`2500210`), la que usan los elementos en
    /// [`Element::article`].
    pub articles: BTreeMap<String, Article>,
    /// Grupos de puentes: hijos nombrados por el object id de la conexión.
    pub bridge_groups: Vec<BridgeGroup>,
}

/// Cabecera `WriterHeader`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Writer {
    /// Usuario que exportó.
    pub name: Option<String>,
    pub vendor: Option<String>,
    /// `2025.0.3.23756`
    pub version: Option<String>,
    pub release: Option<String>,
    /// `2026-07-16`
    pub date: Option<String>,
}

/// `InternalElement` de la jerarquía de instancias.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Element {
    /// GUID CAEX.
    pub id: String,
    /// En las piezas, la designación (`+A1-XD2*2`).
    pub name: String,
    /// Índice del padre en [`Aml::elements`].
    pub parent: Option<usize>,
    /// Profundidad (0 = proyecto).
    pub depth: usize,
    /// Rol ProPanel (`Mounting surface`, `Part placement, terminal`…).
    pub role: Option<String>,
    pub eclass: Option<Eclass>,
    /// `Object identification` (= ep.2000 del manifest).
    pub oid: Option<String>,
    /// `Part number` (ep.22001): `RIT.2500210`.
    pub part_number: Option<String>,
    /// Clave del artículo en [`Aml::articles`] (`RefBaseSystemUnitPath`).
    pub article: Option<String>,
    /// Posición relativa al elemento padre (mm y grados). No coincide con
    /// la transformación del E3D (las mallas tienen otro origen).
    pub frame: Option<Frame>,
    /// Geometría si el elemento es una superficie de montaje.
    pub surface: Option<Surface>,
    /// `Full item designation`: `S1:Frontal de placa de montaje`.
    pub full_designation: Option<String>,
    /// `Location designation with preceding sign` (ep.1240): `+A1`.
    pub location: Option<String>,
    pub location_description: Option<MultiText>,
    /// `Function designation with preceding sign` (ep.1140): `=E01`.
    pub function: Option<String>,
    pub function_description: Option<MultiText>,
    /// `Function text (of main function)` (ep.20463).
    pub function_text: Option<MultiText>,
    /// `Placement` (ep.19007): `=E01+A1&EFS1/ 1.2`.
    pub placement: Option<String>,
    /// ep.20188: `8/1/1`.
    pub function_definition: Option<String>,
    /// `Technical characteristics`: `10...16A`.
    pub technical: Option<String>,
    /// `Layout space name` en los espacios de montaje.
    pub layout_space: Option<String>,
    pub dimensions: Dimensions,
    pub mounting: Mounting,
    pub connectors: Vec<Connector>,
    /// Interfaces de comunicación (`PhysicalEndPoint`…).
    pub interfaces: Vec<String>,
    /// Superficie de montaje sobre la que está (el ancestro más cercano con
    /// rol `Mounting surface`).
    pub mounted_on: Option<usize>,
    /// Espacio de montaje (`Layout space`) al que pertenece.
    pub space: Option<usize>,
}

/// Artículo (`SystemUnitClass`).
#[derive(Debug, Clone, Default, Serialize)]
pub struct Article {
    pub eclass: Option<Eclass>,
    pub manufacturer: Option<String>,
    /// `product article number of manufacturer`: `2500210`.
    pub manufacturer_number: Option<String>,
    /// `Product type`: `SZ.2500210`.
    pub product_type: Option<String>,
    /// `Manufacturer product designation`.
    pub designation: Option<MultiText>,
    /// `Manufacturer product description`.
    pub description: Option<MultiText>,
    pub gtin: Option<String>,
    /// `Weight` (ep.22046), kg.
    pub weight: Option<f64>,
    /// `Address of additional link`: fichas del fabricante.
    pub links: Vec<Link>,
    pub dimensions: Dimensions,
    pub mounting: Mounting,
    pub connectors: Vec<Connector>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Eclass {
    /// `27182808`
    pub code: Option<String>,
    /// `27-18-28-08 Electrical cabinet luminaire (electrical cabinet)`
    pub label: Option<String>,
    pub irdi: Option<String>,
    /// `ECLASS-13.0`
    pub standard: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Link {
    pub url: String,
    pub label: Option<String>,
}

/// Posición: x, y, z (mm) y rotaciones rx, ry, rz (grados).
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub struct Frame {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub rx: f64,
    pub ry: f64,
    pub rz: f64,
}

/// Superficie de montaje ProPanel.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub struct Surface {
    /// Origen en el sistema de la propia superficie (mm).
    pub start: [f64; 3],
    /// Ancho y alto (mm).
    pub size: [f64; 2],
    /// Normal y eje x en coordenadas del espacio de montaje.
    pub normal: [f64; 3],
    pub x_axis: [f64; 3],
    /// Normal y eje x relativos al elemento padre.
    pub normal_relative: [f64; 3],
    pub x_axis_relative: [f64; 3],
}

/// Dimensiones (mm). Las piezas y los artículos usan nombres distintos
/// (`Width` / `width`); aquí van juntas.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Dimensions {
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub depth: Option<f64>,
    pub length: Option<f64>,
    /// `Diameter` de los taladros.
    pub diameter: Option<f64>,
    /// `Nominal diameter` de los taladros roscados: `M5`.
    pub thread: Option<String>,
}

/// Punto con dirección, en el sistema de la pieza (mm).
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub struct Point {
    pub position: [f64; 3],
    pub direction: [f64; 3],
}

/// Datos de montaje (`CAx basic`).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Mounting {
    pub fixing_variants: Vec<FixingVariant>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct FixingVariant {
    /// Número de la variante (`Fixing variant*2` → 2; sin número → 1).
    pub index: u32,
    /// `SIE.3RV2_HSQUER`
    pub name: Option<String>,
    /// `Fixing supply point (in mm)`
    pub supply_point: Option<Point>,
    pub mounting_points: Vec<MountingPoint>,
    /// Recortes de montaje (contornos a mecanizar).
    pub cutouts: Vec<Cutout>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct MountingPoint {
    pub index: u32,
    pub name: Option<String>,
    pub position: Option<Point>,
    /// Ejecuciones posibles (`Type of connection point*n`).
    pub fixings: Vec<Fixing>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Fixing {
    pub index: u32,
    /// `Drill hole`, `Threaded hole`…
    pub execution: Option<String>,
    /// `Hole, diameter` (mm).
    pub hole_diameter: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Cutout {
    pub index: u32,
    pub contours: Vec<Contour>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Contour {
    pub index: u32,
    /// `Type of contour`
    pub kind: Option<String>,
    pub start: [f64; 3],
    pub end: [f64; 3],
}

/// Conector (`Connector*n`): punto de enchufe de la pieza.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Connector {
    pub index: u32,
    pub identification: Option<String>,
    pub position: Option<Point>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BridgeGroup {
    /// Nombre del grupo (`Plug-in bridge`, `Saddle jumper`…).
    pub kind: String,
    /// Object ids (`253/18/64304/0`) de las conexiones-puente.
    pub object_ids: Vec<String>,
}

impl Element {
    /// Clave `typeId/objectId` (`180/50377`), la de las piezas del E3D.
    pub fn key(&self) -> Option<&str> {
        object_key(self.oid.as_deref()?)
    }

    pub fn is_mounting_surface(&self) -> bool {
        self.role.as_deref() == Some("Mounting surface")
    }
}

/// `253/180/50377/0` → `180/50377`.
pub fn object_key(oid: &str) -> Option<&str> {
    let start = oid.find('/')? + 1;
    let second = start + oid[start..].find('/')? + 1;
    let end = oid[second..].find('/').map_or(oid.len(), |i| second + i);
    Some(&oid[start..end])
}

impl Aml {
    /// Lee el AML desde los bytes del fichero (UTF-8, con o sin BOM).
    pub fn parse(data: &[u8]) -> Result<Self> {
        let data = data.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(data);
        let text = std::str::from_utf8(data).map_err(|_| Error::Missing("AML en UTF-8"))?;
        Parser::default().run(text)
    }

    pub fn article_of(&self, e: &Element) -> Option<&Article> {
        self.articles.get(e.article.as_deref()?)
    }

    /// Hijos directos de un elemento.
    pub fn children(&self, i: usize) -> impl Iterator<Item = usize> + '_ {
        // Los elementos están en orden de documento: los descendientes van
        // justo detrás del padre.
        let depth = self.elements[i].depth;
        self.elements[i + 1..]
            .iter()
            .take_while(move |e| e.depth > depth)
            .enumerate()
            .filter(move |(_, e)| e.parent == Some(i))
            .map(move |(k, _)| i + 1 + k)
    }

    /// Descendientes de un elemento, en orden de documento.
    pub fn descendants(&self, i: usize) -> impl Iterator<Item = usize> + '_ {
        let depth = self.elements[i].depth;
        let n = self.elements[i + 1..]
            .iter()
            .take_while(|e| e.depth > depth)
            .count();
        i + 1..i + 1 + n
    }

    /// Placa (o puerta, perfil, lateral…) sobre la que está montado un
    /// elemento: el que lleva su superficie de montaje.
    pub fn plate_of(&self, i: usize) -> Option<usize> {
        self.elements[self.elements[i].mounted_on?].parent
    }

    /// Índice de elementos por clave `typeId/objectId`.
    pub fn by_key(&self) -> HashMap<&str, usize> {
        self.elements
            .iter()
            .enumerate()
            .filter_map(|(i, e)| Some((e.key()?, i)))
            .collect()
    }

    /// Enlaza cada elemento con su función del manifest por
    /// `Object identification` = ep.2000. Devuelve índice → id de función.
    pub fn link_functions(&self, project: &Project) -> BTreeMap<usize, i64> {
        let by_oid: HashMap<&str, i64> = project
            .functions
            .iter()
            .filter_map(|f| Some((f.oid.as_deref()?, f.id)))
            .collect();
        self.elements
            .iter()
            .enumerate()
            .filter_map(|(i, e)| Some((i, *by_oid.get(e.oid.as_deref()?)?)))
            .collect()
    }

    fn finish(&mut self) {
        // Superficie de montaje y espacio de cada elemento: se heredan del
        // padre (los padres van antes en el vector).
        for i in 0..self.elements.len() {
            let (mounted_on, space) = match self.elements[i].parent {
                Some(p) => {
                    let parent = &self.elements[p];
                    let surface = if parent.is_mounting_surface() {
                        Some(p)
                    } else {
                        parent.mounted_on
                    };
                    let space = if parent.layout_space.is_some() {
                        Some(p)
                    } else {
                        parent.space
                    };
                    (surface, space)
                }
                None => (None, None),
            };
            self.elements[i].mounted_on = mounted_on;
            self.elements[i].space = space;
        }
        // Grupos de puentes: elementos de primer nivel sin espacio de montaje
        // cuyos hijos se llaman por object id (`253/18/64304/0`).
        for i in 0..self.elements.len() {
            let g = &self.elements[i];
            if g.depth != 1 || g.layout_space.is_some() {
                continue;
            }
            let object_ids: Vec<String> = self
                .children(i)
                .map(|c| &self.elements[c].name)
                .filter(|n| is_object_id(n))
                .cloned()
                .collect();
            if !object_ids.is_empty() {
                self.bridge_groups.push(BridgeGroup {
                    kind: g.name.clone(),
                    object_ids,
                });
            }
        }
    }
}

fn is_object_id(s: &str) -> bool {
    s.contains('/')
        && s.split('/')
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

/// Dónde se escriben los `<Value>` del contenedor actual.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Owner {
    /// Elemento de la jerarquía de instancias (índice).
    Element(usize),
    /// Artículo en construcción.
    Article,
    /// Contenedor que no interesa (clases de rol, de interfaz…).
    Ignored,
}

#[derive(Default)]
struct Parser {
    aml: Aml,
    /// Contenedores abiertos (`InternalElement`, `SystemUnitClass`).
    owners: Vec<Owner>,
    /// Elementos de la jerarquía abiertos (para el padre).
    open: Vec<usize>,
    /// Nombres de los `<Attribute>` abiertos.
    attrs: Vec<String>,
    /// Artículo en construcción y su clave.
    article: Option<(String, Article)>,
    in_instances: bool,
    in_articles: bool,
    /// Texto de `<Value>` o de un campo de `WriterHeader` en curso.
    text: Option<String>,
    /// Etiqueta de `WriterHeader` en curso.
    writer_field: Option<String>,
    languages: BTreeSet<String>,
}

impl Parser {
    fn run(mut self, xml: &str) -> Result<Aml> {
        let mut reader = quick_xml::Reader::from_str(xml);
        loop {
            match reader.read_event()? {
                Event::Start(e) => self.start(&e, false)?,
                Event::Empty(e) => self.start(&e, true)?,
                Event::End(e) => self.end(e.local_name().as_ref()),
                Event::Text(t) => {
                    if let Some(buf) = &mut self.text {
                        buf.push_str(&t.xml10_content());
                    }
                }
                Event::GeneralRef(r) => {
                    if let Some(buf) = &mut self.text {
                        if let Some(c) = r.resolve_char_ref()? {
                            buf.push(c);
                        } else if let Some(s) =
                            quick_xml::escape::resolve_xml_entity(&r.into_inner())
                        {
                            buf.push_str(s);
                        }
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }
        let mut aml = self.aml;
        aml.languages = self.languages.into_iter().collect();
        aml.finish();
        Ok(aml)
    }

    fn owner(&self) -> Owner {
        self.owners.last().copied().unwrap_or(Owner::Ignored)
    }

    fn start(&mut self, e: &BytesStart, empty: bool) -> Result<()> {
        let tag = e.local_name();
        match tag.as_ref() {
            "InstanceHierarchy" if !empty => self.in_instances = true,
            "SystemUnitClassLib" if !empty => self.in_articles = true,
            "InternalElement" if self.in_instances => {
                let parent = self.open.last().copied();
                let element = Element {
                    id: attr(e, "ID")?.unwrap_or_default(),
                    name: attr(e, "Name")?.unwrap_or_default(),
                    parent,
                    depth: self.open.len(),
                    article: attr(e, "RefBaseSystemUnitPath")?.map(|p| article_key(&p)),
                    ..Element::default()
                };
                if parent.is_none() && self.aml.project_name.is_none() {
                    self.aml.project_name = Some(element.name.clone());
                }
                self.aml.elements.push(element);
                let i = self.aml.elements.len() - 1;
                if !empty {
                    self.open.push(i);
                    self.owners.push(Owner::Element(i));
                }
            }
            // Los artículos de primer nivel; las clases anidadas se ignoran.
            "SystemUnitClass" if self.in_articles && !empty => {
                if self.article.is_none() {
                    let key = article_key(&attr(e, "Name")?.unwrap_or_default());
                    self.article = Some((key, Article::default()));
                    self.owners.push(Owner::Article);
                } else {
                    self.owners.push(Owner::Ignored);
                }
            }
            "InternalElement" | "SystemUnitClass" if !empty => self.owners.push(Owner::Ignored),
            "Attribute" if !empty => self.attrs.push(attr(e, "Name")?.unwrap_or_default()),
            "Value" if !empty => self.text = Some(String::new()),
            "SupportedRoleClass" | "RoleRequirements" => {
                let path = match attr(e, "RefRoleClassPath")? {
                    Some(p) => Some(p),
                    None => attr(e, "RefBaseRoleClassPath")?,
                };
                if let Some(path) = path {
                    self.role(&path);
                }
            }
            "ExternalInterface" => {
                if let (Owner::Element(i), Some(name)) = (self.owner(), attr(e, "Name")?)
                    && INTERFACES.contains(&name.as_str())
                {
                    let list = &mut self.aml.elements[i].interfaces;
                    if !list.contains(&name) {
                        list.push(name);
                    }
                }
            }
            "WriterName"
            | "WriterVendor"
            | "WriterVersion"
            | "WriterRelease"
            | "LastWritingDateTime"
                if !empty =>
            {
                self.writer_field = Some(tag.as_ref().to_owned());
                self.text = Some(String::new());
            }
            _ => {}
        }
        Ok(())
    }

    fn end(&mut self, tag: &str) {
        match tag {
            "InstanceHierarchy" => self.in_instances = false,
            "SystemUnitClassLib" => self.in_articles = false,
            "InternalElement" | "SystemUnitClass" => match self.owners.pop() {
                Some(Owner::Element(_)) => {
                    self.open.pop();
                }
                Some(Owner::Article) => {
                    if let Some((key, article)) = self.article.take() {
                        self.aml.articles.insert(key, article);
                    }
                }
                _ => {}
            },
            "Attribute" => {
                self.attrs.pop();
            }
            "Value" => {
                if let Some(value) = self.text.take() {
                    self.value(value);
                }
            }
            "WriterName"
            | "WriterVendor"
            | "WriterVersion"
            | "WriterRelease"
            | "LastWritingDateTime" => {
                let value = self.text.take().map(|v| v.trim().to_owned());
                let w = &mut self.aml.writer;
                match self.writer_field.take().as_deref() {
                    Some("WriterName") => w.name = value,
                    Some("WriterVendor") => w.vendor = value,
                    Some("WriterVersion") => w.version = value,
                    Some("WriterRelease") => w.release = value,
                    Some("LastWritingDateTime") => w.date = value,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn role(&mut self, path: &str) {
        let eclass = path.contains("EclassRoleClassLib");
        let propanel = path.contains("EPLANProPanelRoleClassLib");
        let leaf = path.rsplit('/').next().unwrap_or(path);
        let leaf = leaf
            .trim_start_matches('[')
            .trim_end_matches(']')
            .to_owned();
        let mut no_role = None;
        let (role, class) = match self.owner() {
            Owner::Element(i) => {
                let e = &mut self.aml.elements[i];
                (&mut e.role, &mut e.eclass)
            }
            Owner::Article => match self.article.as_mut() {
                Some((_, a)) => (&mut no_role, &mut a.eclass),
                None => return,
            },
            Owner::Ignored => return,
        };
        if propanel && role.is_none() {
            *role = Some(leaf);
        } else if eclass {
            let c = class.get_or_insert_with(Eclass::default);
            if c.label.is_none() {
                c.label = eclass_leaf(path);
            }
        }
    }

    fn value(&mut self, value: String) {
        if self.attrs.is_empty() {
            return;
        }
        let path: Vec<&str> = self.attrs.iter().map(String::as_str).collect();
        // Textos multilingües: `<Attribute Name="aml-lang=de-DE">` dentro del
        // atributo de texto.
        let (path, lang) = match path.split_last() {
            Some((last, rest)) if last.starts_with("aml-lang=") => {
                let lang = &last["aml-lang=".len()..];
                self.languages.insert(lang.to_owned());
                (rest, lang)
            }
            _ => (path.as_slice(), ""),
        };
        let value = value.trim();
        if value.is_empty() {
            return;
        }
        match self.owner() {
            Owner::Element(i) => {
                if self.aml.elements[i].depth == 0 {
                    if path == ["Project description"] {
                        let d = self
                            .aml
                            .project_description
                            .get_or_insert_with(MultiText::new);
                        d.insert(lang.to_owned(), value.to_owned());
                    }
                    return;
                }
                element_value(&mut self.aml.elements[i], path, lang, value);
            }
            Owner::Article => {
                if let Some((_, a)) = &mut self.article {
                    article_value(a, path, lang, value);
                }
            }
            Owner::Ignored => {}
        }
    }
}

/// Interfaces que se conservan (el resto, p. ej. `DrillPattern`, no).
const INTERFACES: &[&str] = &[
    "PhysicalEndPoint",
    "LogicInterface",
    "VariableInterface",
    "Communication",
    "PortConnector",
];

fn attr(e: &BytesStart, name: &str) -> Result<Option<String>> {
    match e.try_get_attribute(name)? {
        Some(a) => Ok(Some(
            a.normalized_value(XmlVersion::Implicit1_0)?.into_owned(),
        )),
        None => Ok(None),
    }
}

/// `ProductionDataSystemUnitClass/[2500210]` o `[2500210]` → `2500210`.
fn article_key(path: &str) -> String {
    let leaf = path.rsplit('/').next().unwrap_or(path);
    leaf.trim_start_matches('[')
        .trim_end_matches(']')
        .to_owned()
}

/// Último segmento legible de una ruta eCl@ss: `[27-25-01-01 Feed-through…]`.
fn eclass_leaf(path: &str) -> Option<String> {
    let end = path.rfind(']')?;
    let start = path[..end].rfind('[')?;
    Some(path[start + 1..end].to_owned())
}

/// `Fixing variant*2` con prefijo `Fixing variant` → 2; sin número → 1.
fn indexed(name: &str, prefix: &str) -> Option<u32> {
    let rest = name.strip_prefix(prefix)?;
    if rest.is_empty() {
        return Some(1);
    }
    rest.trim_start_matches('*').parse().ok()
}

fn num(v: &str) -> Option<f64> {
    v.replace(',', ".")
        .parse()
        .ok()
        .filter(|x: &f64| x.is_finite())
}

/// Busca o crea el elemento con ese índice.
fn slot<T: Default>(
    list: &mut Vec<T>,
    index: u32,
    get: impl Fn(&T) -> u32,
    set: impl Fn(&mut T, u32),
) -> &mut T {
    match list.iter().position(|x| get(x) == index) {
        Some(p) => &mut list[p],
        None => {
            let mut item = T::default();
            set(&mut item, index);
            list.push(item);
            list.last_mut().expect("recién añadido")
        }
    }
}

fn multi(target: &mut Option<MultiText>, lang: &str, value: &str) {
    target
        .get_or_insert_with(MultiText::new)
        .insert(lang.to_owned(), value.to_owned());
}

/// `AXIS_X`… dentro de un punto con dirección.
fn point_value(p: &mut Option<Point>, axis: &str, value: &str) {
    let Some(v) = num(value) else { return };
    let p = p.get_or_insert_with(Point::default);
    match axis {
        "AXIS_X" => p.position[0] = v,
        "AXIS_Y" => p.position[1] = v,
        "AXIS_Z" => p.position[2] = v,
        "AXIS_DIRECTION_X" => p.direction[0] = v,
        "AXIS_DIRECTION_Y" => p.direction[1] = v,
        "AXIS_DIRECTION_Z" => p.direction[2] = v,
        _ => {}
    }
}

fn xyz(target: &mut [f64; 3], axis: &str, value: &str) {
    let k = match axis {
        "X" | "AXIS_DIRECTION_X" | "AXIS_X" => 0,
        "Y" | "AXIS_DIRECTION_Y" | "AXIS_Y" => 1,
        "Z" | "AXIS_DIRECTION_Z" | "AXIS_Z" => 2,
        _ => return,
    };
    if let Some(v) = num(value) {
        target[k] = v;
    }
}

/// Atributos comunes a elementos y artículos. Devuelve `true` si lo ha
/// consumido.
fn common_value(
    eclass: &mut Option<Eclass>,
    dims: &mut Dimensions,
    mounting: &mut Mounting,
    connectors: &mut Vec<Connector>,
    path: &[&str],
    value: &str,
) -> bool {
    match path {
        ["ClassificationClass"] => {
            eclass.get_or_insert_with(Eclass::default).code = Some(value.into());
        }
        ["IRDI"] => eclass.get_or_insert_with(Eclass::default).irdi = Some(value.into()),
        ["Standard"] => eclass.get_or_insert_with(Eclass::default).standard = Some(value.into()),
        ["Diameter"] => dims.diameter = num(value),
        ["Nominal diameter"] => dims.thread = Some(value.into()),
        [.., last] if path.iter().all(|p| !p.contains('*') && *p != "CAx basic") => {
            match last.to_ascii_lowercase().as_str() {
                "width" => dims.width = num(value),
                "height" => dims.height = num(value),
                "depth" => dims.depth = num(value),
                "length" => dims.length = num(value),
                _ => return false,
            }
        }
        ["CAx basic", variant, rest @ ..] => {
            let Some(vi) = indexed(variant, "Fixing variant") else {
                return false;
            };
            let fv = slot(
                &mut mounting.fixing_variants,
                vi,
                |x| x.index,
                |x, i| x.index = i,
            );
            match rest {
                ["name of fixing variant"] => fv.name = Some(value.into()),
                ["Fixing supply point (in mm)", axis] => {
                    point_value(&mut fv.supply_point, axis, value)
                }
                [mp, rest @ ..] if indexed(mp, "Mounting point").is_some() => {
                    let mi = indexed(mp, "Mounting point").unwrap_or(1);
                    let mp = slot(&mut fv.mounting_points, mi, |x| x.index, |x, i| x.index = i);
                    match rest {
                        ["mounting point name"] => mp.name = Some(value.into()),
                        ["mounting point (in mm)", axis] => {
                            point_value(&mut mp.position, axis, value)
                        }
                        [kind, field] if indexed(kind, "Type of connection point").is_some() => {
                            let ki = indexed(kind, "Type of connection point").unwrap_or(1);
                            let f = slot(&mut mp.fixings, ki, |x| x.index, |x, i| x.index = i);
                            match *field {
                                "Execution of the fixing point" => f.execution = Some(value.into()),
                                "Hole, diameter" => f.hole_diameter = num(value),
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                [cut, contour, rest @ ..] if indexed(cut, "Mounting cut-out").is_some() => {
                    let Some(ci) = indexed(contour, "Contour") else {
                        return true;
                    };
                    let cutout = slot(
                        &mut fv.cutouts,
                        indexed(cut, "Mounting cut-out").unwrap_or(1),
                        |x| x.index,
                        |x, i| x.index = i,
                    );
                    let c = slot(&mut cutout.contours, ci, |x| x.index, |x, i| x.index = i);
                    match rest {
                        ["Type of contour"] => c.kind = Some(value.into()),
                        ["Starting point (in mm)", axis] => xyz(&mut c.start, axis, value),
                        ["Final point (in mm)", axis] => xyz(&mut c.end, axis, value),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        [conn, rest @ ..] if indexed(conn, "Connector").is_some() => {
            let ci = indexed(conn, "Connector").unwrap_or(1);
            let c = slot(connectors, ci, |x| x.index, |x, i| x.index = i);
            match rest {
                ["Connector identification"] => c.identification = Some(value.into()),
                ["Position (in mm)", axis] => point_value(&mut c.position, axis, value),
                _ => {}
            }
        }
        _ => return false,
    }
    true
}

fn element_value(e: &mut Element, path: &[&str], lang: &str, value: &str) {
    match path {
        ["Frame", axis] => {
            let Some(v) = num(value) else { return };
            let f = e.frame.get_or_insert_with(Frame::default);
            match *axis {
                "x" => f.x = v,
                "y" => f.y = v,
                "z" => f.z = v,
                "rx" => f.rx = v,
                "ry" => f.ry = v,
                "rz" => f.rz = v,
                _ => {}
            }
        }
        ["Object identification"] => e.oid = Some(value.into()),
        ["Part number"] => {
            if e.part_number.is_none() {
                e.part_number = Some(value.into());
            }
        }
        ["Full item designation"] => e.full_designation = Some(value.into()),
        ["Location designation with preceding sign"] => e.location = Some(value.into()),
        ["Location designation : Description"] => multi(&mut e.location_description, lang, value),
        ["Function designation with preceding sign"] => e.function = Some(value.into()),
        ["Function designation: Description"] => multi(&mut e.function_description, lang, value),
        ["Function text (of main function)"] => multi(&mut e.function_text, lang, value),
        ["Placement"] => e.placement = Some(value.into()),
        ["Function definition : Category / Group / Definition"] => {
            e.function_definition = Some(value.into());
        }
        ["Technical characteristics"] => e.technical = Some(value.into()),
        ["Layout space name"] => e.layout_space = Some(value.into()),
        [group, axis] if is_surface_group(group) => {
            let s = e.surface.get_or_insert_with(Surface::default);
            match *group {
                "Startpoint of mounting surface" => xyz(&mut s.start, axis, value),
                "Size of mounting surface" => {
                    let mut v = [s.size[0], s.size[1], 0.0];
                    xyz(&mut v, axis, value);
                    s.size = [v[0], v[1]];
                }
                "Absolute direction of surface normal" => xyz(&mut s.normal, axis, value),
                "Absolute direction of surface x axis" => xyz(&mut s.x_axis, axis, value),
                "Relative direction of surface normal" => xyz(&mut s.normal_relative, axis, value),
                "Relative direction of surface x axis" => xyz(&mut s.x_axis_relative, axis, value),
                _ => {}
            }
        }
        _ => {
            common_value(
                &mut e.eclass,
                &mut e.dimensions,
                &mut e.mounting,
                &mut e.connectors,
                path,
                value,
            );
        }
    }
}

fn is_surface_group(name: &str) -> bool {
    matches!(
        name,
        "Startpoint of mounting surface"
            | "Size of mounting surface"
            | "Absolute direction of surface normal"
            | "Absolute direction of surface x axis"
            | "Relative direction of surface normal"
            | "Relative direction of surface x axis"
    )
}

fn article_value(a: &mut Article, path: &[&str], lang: &str, value: &str) {
    match path {
        ["Identification", "Manufacturer", field] => match *field {
            "Manufacturer name" => a.manufacturer = Some(value.into()),
            "product article number of manufacturer" => a.manufacturer_number = Some(value.into()),
            "Product type" => a.product_type = Some(value.into()),
            "Manufacturer product designation" => multi(&mut a.designation, lang, value),
            "Manufacturer product description" => multi(&mut a.description, lang, value),
            "GTIN" => a.gtin = Some(value.into()),
            _ => {}
        },
        ["Weight"] => a.weight = num(value),
        ["Information", "Address of additional link"] => a.links = parse_links(value),
        _ => {
            common_value(
                &mut a.eclass,
                &mut a.dimensions,
                &mut a.mounting,
                &mut a.connectors,
                path,
                value,
            );
        }
    }
}

/// Una línea por enlace: `url<TAB>etiqueta`.
fn parse_links(value: &str) -> Vec<Link> {
    value
        .lines()
        .filter_map(|line| {
            let (url, label) = line.split_once('\t').unwrap_or((line, ""));
            let url = url.trim();
            (!url.is_empty()).then(|| Link {
                url: url.to_owned(),
                label: Some(label.trim())
                    .filter(|l| !l.is_empty())
                    .map(str::to_owned),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<CAEXFile>
 <AdditionalInformation><WriterHeader><WriterVersion>2025.0.3</WriterVersion></WriterHeader></AdditionalInformation>
 <InstanceHierarchy Name="P">
  <InternalElement Name="P" ID="1">
   <Attribute Name="Project description"><Value>Rectificadora</Value>
    <Attribute Name="aml-lang=en-US"><Value>Grinding &amp; more</Value></Attribute>
   </Attribute>
   <InternalElement Name="A1" ID="2">
    <Attribute Name="Layout space name"><Value>A1</Value></Attribute>
    <Attribute Name="Object identification"><Value>253/183/38195/0</Value></Attribute>
    <InternalElement Name="+A1-U3*1" ID="3">
     <SupportedRoleClass RefRoleClassPath="EPLANProPanelRoleClassLib/Mounting surface"/>
     <Attribute Name="Size of mounting surface"><Attribute Name="X"><Value>699</Value></Attribute><Attribute Name="Y"><Value>1696</Value></Attribute></Attribute>
     <InternalElement Name="=E01+A1-EA1" ID="4" RefBaseSystemUnitPath="Lib/[2500210]">
      <SupportedRoleClass RefRoleClassPath="[EPLANProPanelRoleClassLib]/[Part placement, general device]"/>
      <SupportedRoleClass RefRoleClassPath="[EclassRoleClassLib]/[X]/[27-18-28-08 Luminaire]"/>
      <Attribute Name="ClassificationClass"><Value>27182808</Value></Attribute>
      <Attribute Name="Frame"><Attribute Name="x"><Value>130.85</Value></Attribute><Attribute Name="rz"><Value>-90</Value></Attribute></Attribute>
      <Attribute Name="Object identification"><Value>253/180/50377/0</Value></Attribute>
      <Attribute Name="Part number"><Value>RIT.2500210</Value></Attribute>
     </InternalElement>
    </InternalElement>
   </InternalElement>
  </InternalElement>
 </InstanceHierarchy>
 <SystemUnitClassLib Name="Lib">
  <SystemUnitClass Name="2500210">
   <Attribute Name="Identification"><Attribute Name="Manufacturer">
    <Attribute Name="Manufacturer name"><Value>Rittal</Value></Attribute>
    <Attribute Name="GTIN"><Value>4028177000000</Value></Attribute>
   </Attribute></Attribute>
   <Attribute Name="Weight"><Value>0.614</Value></Attribute>
   <Attribute Name="Information"><Attribute Name="Address of additional link"><Value>http://a/1	Doc ES
http://a/2	</Value></Attribute></Attribute>
   <Attribute Name="CAx basic"><Attribute Name="Fixing variant*1">
    <Attribute Name="Mounting point*2">
     <Attribute Name="mounting point (in mm)"><Attribute Name="AXIS_X"><Value>10</Value></Attribute></Attribute>
     <Attribute Name="Type of connection point*1"><Attribute Name="Hole, diameter"><Value>5.5</Value></Attribute></Attribute>
    </Attribute>
   </Attribute></Attribute>
  </SystemUnitClass>
 </SystemUnitClassLib>
</CAEXFile>"#;

    #[test]
    fn parses_tree_and_articles() {
        let aml = Aml::parse(XML.as_bytes()).unwrap();
        assert_eq!(aml.writer.version.as_deref(), Some("2025.0.3"));
        assert_eq!(aml.project_name.as_deref(), Some("P"));
        let d = aml.project_description.as_ref().unwrap();
        assert_eq!(d["en-US"], "Grinding & more");
        assert_eq!(aml.languages, ["en-US"]);
        let ea1 = &aml.elements[3];
        assert_eq!(ea1.role.as_deref(), Some("Part placement, general device"));
        assert_eq!(ea1.key(), Some("180/50377"));
        assert_eq!(ea1.frame.unwrap().x, 130.85);
        assert_eq!(ea1.frame.unwrap().rz, -90.0);
        assert_eq!(ea1.mounted_on, Some(2));
        assert_eq!(ea1.space, Some(1));
        let eclass = ea1.eclass.as_ref().unwrap();
        assert_eq!(eclass.code.as_deref(), Some("27182808"));
        assert_eq!(eclass.label.as_deref(), Some("27-18-28-08 Luminaire"));
        assert_eq!(aml.elements[2].surface.unwrap().size, [699.0, 1696.0]);
        let a = aml.article_of(ea1).unwrap();
        assert_eq!(a.manufacturer.as_deref(), Some("Rittal"));
        assert_eq!(a.weight, Some(0.614));
        assert_eq!(a.links.len(), 2);
        assert_eq!(a.links[0].label.as_deref(), Some("Doc ES"));
        let mp = &a.mounting.fixing_variants[0].mounting_points[0];
        assert_eq!(mp.index, 2);
        assert_eq!(mp.position.unwrap().position[0], 10.0);
        assert_eq!(mp.fixings[0].hole_diameter, Some(5.5));
        assert_eq!(aml.children(1).collect::<Vec<_>>(), [2]);
        assert_eq!(aml.descendants(1).collect::<Vec<_>>(), [2, 3]);
    }

    #[test]
    fn object_keys() {
        assert_eq!(object_key("253/180/50377/0"), Some("180/50377"));
        assert_eq!(object_key("180/50377"), None);
        assert!(is_object_id("253/18/64304/0"));
        assert!(!is_object_id("+A1-U3"));
    }
}
