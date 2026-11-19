//! Formato binario E3D de EPLAN (versiones 1 a 4): el modelo 3D de un
//! espacio de instalación.
//!
//! El formato y lo que se sabe de cada campo están en `docs/formats.md`
//! ("EPLAN E3D" y "Observed semantics"). En resumen:
//!
//! - Little-endian. Cabecera con la versión, la cámara inicial
//!   (`viewTransform`), `viewParams` (sin significado conocido) y los colores
//!   de fondo; luego luces, texturas, mallas y piezas.
//! - Cada pieza es una instancia de una malla con su transformación, que ya la
//!   coloca en coordenadas del mundo (mm, Z hacia arriba). La cámara **no** se
//!   aplica a la geometría.
//! - Desde la v3 cada pieza lleva `typeId/objectId`, que la enlaza con el
//!   manifest (`…/<typeId>/<objectId>/…` en ep.2000) y con el AML
//!   (`Object identification`).
//!
//! Un archivo corrupto o malicioso no puede agotar la memoria: cada longitud
//! se comprueba contra los bytes que quedan antes de reservar nada, y el
//! tamaño total está limitado por [`Limits`].

use std::borrow::Cow;

use serde::Serialize;

/// Matriz 4x4 *column-major* (la convención de WebGL y three.js).
pub type Mat4 = [f32; 16];
pub type Vec3 = [f32; 3];
/// RGBA en `[0, 1]`. El alfa ya es opacidad (el fichero guarda transparencia).
pub type Rgba = [f32; 4];

/// Última versión del formato que se sabe leer.
pub const MAX_SUPPORTED_VERSION: u8 = 4;

/// Error de lectura de un E3D, con la posición en bytes donde ocurrió.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum E3dError {
    #[error("versión de E3D no soportada: {0} (se leen 1 a {MAX_SUPPORTED_VERSION})")]
    UnsupportedVersion(u8),
    #[error("fin de fichero inesperado en el byte {at}: faltan {needed} bytes")]
    UnexpectedEof { at: usize, needed: usize },
    #[error("{what} = {value} en el byte {at} no es válido")]
    Invalid {
        what: &'static str,
        value: i64,
        at: usize,
    },
    #[error("el fichero ocupa {size} bytes, más que el límite de {limit}")]
    TooLarge { size: usize, limit: usize },
    #[error("sobran {0} bytes al final del fichero")]
    TrailingData(usize),
}

/// Límites de lectura. Los valores por defecto sobran para cualquier
/// armario real (el mayor E3D del ejemplo ocupa 8 MB).
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Tamaño máximo del fichero en bytes.
    pub max_file_size: usize,
    /// Número máximo de elementos de cualquier array.
    pub max_array_len: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_file_size: 512 << 20,
            max_array_len: 1 << 24,
        }
    }
}

/// Escena completa de un E3D.
#[derive(Debug, Clone, Serialize)]
pub struct Scene {
    pub version: u8,
    pub view: View,
    pub lights: Vec<Light>,
    pub textures: Vec<Texture>,
    pub meshes: Vec<Mesh>,
    pub parts: Vec<Part>,
}

/// Cabecera de la vista.
#[derive(Debug, Clone, Serialize)]
pub struct View {
    /// `viewTransform`: cámara inicial (rotación isométrica + traslación).
    /// No se aplica a la geometría.
    pub camera: Mat4,
    /// `viewParams`: significado desconocido, se guarda en bruto.
    pub params: Vec3,
    pub background_top: Rgba,
    pub background_bottom: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LightKind {
    Ambient,
    Directional,
    Point,
    Spot,
}

#[derive(Debug, Clone, Serialize)]
pub struct Light {
    pub kind: LightKind,
    pub color: Rgba,
    pub position: Option<Vec3>,
    pub direction: Option<Vec3>,
    /// Solo focos: ángulo y exponente.
    pub spot: Option<(f32, f32)>,
}

/// Textura sin comprimir, ya en RGBA (el fichero la guarda en BGRA).
#[derive(Debug, Clone, Serialize)]
pub struct Texture {
    pub width: u16,
    pub height: u16,
    #[serde(skip)]
    pub rgba: Vec<u8>,
}

/// Malla: vértices intercalados y grupos de caras y aristas.
#[derive(Debug, Clone, Serialize)]
pub struct Mesh {
    pub has_positions: bool,
    pub has_normals: bool,
    pub has_tex_coords: bool,
    /// Floats por vértice: posición (3), normal (3), uv (2), según los flags.
    pub stride: usize,
    #[serde(skip)]
    pub vertices: Vec<f32>,
    pub faces: Vec<FaceGroup>,
    pub edges: Vec<EdgeGroup>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FaceGroup {
    pub material: Material,
    pub elements: Elements,
}

#[derive(Debug, Clone, Serialize)]
pub struct EdgeGroup {
    pub style: EdgeStyle,
    pub elements: Elements,
}

#[derive(Debug, Clone, Serialize)]
pub struct Material {
    /// Si es `false`, se usa el color de la pieza.
    pub color_valid: bool,
    pub color: Rgba,
    /// Índice en [`Scene::textures`]; `None` sin textura.
    pub texture: Option<u16>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EdgeStyle {
    pub color: Rgba,
    pub line_width: f32,
    pub line_type: u8,
}

/// Primitivas al estilo OpenGL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Primitive {
    Points,
    LineStrip,
    LineLoop,
    Lines,
    TriangleStrip,
    TriangleFan,
    Triangles,
}

#[derive(Debug, Clone, Serialize)]
pub struct Elements {
    /// Solo v2+. No es el centroide del grupo: se guarda en bruto.
    pub center: Option<Vec3>,
    pub mode: Primitive,
    /// Número de índices declarado (en los ficheros vistos, igual a
    /// `indices.len()`).
    pub count: i32,
    #[serde(skip)]
    pub indices: Vec<u16>,
}

/// Pieza: una instancia de malla colocada en el mundo.
#[derive(Debug, Clone, Serialize)]
pub struct Part {
    pub mesh: u16,
    /// Coloca la malla en coordenadas del mundo (mm).
    pub transform: Mat4,
    /// Color de la pieza (para materiales con `color_valid = false`).
    pub color: Rgba,
    /// v3+.
    pub type_id: Option<u16>,
    /// v3+. Junto con `type_id`, enlaza con el manifest y el AML.
    pub object_id: Option<i32>,
    /// v4+: designaciones dibujadas sobre la pieza.
    pub texts: Vec<TextLine>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TextLine {
    pub transform: Mat4,
    pub height: f32,
    /// Código de justificación EPLAN (1–3 abajo, 4–6 centro, 7+ arriba;
    /// en cada fila izquierda, centro, derecha).
    pub justification: i16,
    pub text: String,
}

/// Qué es cada pieza según su `typeId`. Evidencia en `docs/formats.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PartKind {
    /// 166: hilo enrutado (`connection3did` del manifest).
    Wire,
    /// 180: dispositivo, borne o componente de armario.
    Device,
    /// 179: envolvente (armario).
    Enclosure,
    /// 173: canaleta.
    CableDuct,
    /// 178: carril DIN.
    MountingRail,
    /// 174: barra (embarrado).
    Busbar,
    /// 188: sistema de embarrado.
    BusbarSystem,
    /// 189: haz de cobre.
    CopperBundle,
    /// 170: superficie de montaje.
    MountingSurface,
    /// 175: zona restringida de colocación o de recorte (en el AML).
    RestrictedArea,
    /// 176: taladro, taladro roscado o recorte (en el AML).
    Cutout,
    /// Sin identificar (185, 187, 238…) o sin `typeId` (v1–v2).
    Unknown,
}

impl PartKind {
    pub fn from_type_id(type_id: Option<u16>) -> Self {
        match type_id {
            Some(166) => Self::Wire,
            Some(170) => Self::MountingSurface,
            Some(173) => Self::CableDuct,
            Some(174) => Self::Busbar,
            Some(175) => Self::RestrictedArea,
            Some(176) => Self::Cutout,
            Some(178) => Self::MountingRail,
            Some(179) => Self::Enclosure,
            Some(180) => Self::Device,
            Some(188) => Self::BusbarSystem,
            Some(189) => Self::CopperBundle,
            _ => Self::Unknown,
        }
    }

    /// Pieza que existe físicamente en el armario. Las superficies de
    /// montaje, zonas restringidas, recortes y piezas sin identificar son
    /// ayudas de diseño.
    pub fn is_physical(self) -> bool {
        !matches!(
            self,
            Self::MountingSurface | Self::RestrictedArea | Self::Cutout | Self::Unknown
        )
    }
}

impl Part {
    pub fn kind(&self) -> PartKind {
        PartKind::from_type_id(self.type_id)
    }

    /// Clave `typeId/objectId` (`180/50377`), la misma que usa el manifest.
    pub fn key(&self) -> Option<String> {
        Some(format!("{}/{}", self.type_id?, self.object_id?))
    }
}

impl Scene {
    /// Lee un E3D con los límites por defecto.
    pub fn parse(data: &[u8]) -> Result<Self, E3dError> {
        Self::parse_with(data, Limits::default())
    }

    pub fn parse_with(data: &[u8], limits: Limits) -> Result<Self, E3dError> {
        if data.len() > limits.max_file_size {
            return Err(E3dError::TooLarge {
                size: data.len(),
                limit: limits.max_file_size,
            });
        }
        let mut r = Reader {
            data,
            pos: 0,
            limits,
        };
        let version = r.u8()?;
        if !(1..=MAX_SUPPORTED_VERSION).contains(&version) {
            return Err(E3dError::UnsupportedVersion(version));
        }
        let view = View {
            camera: r.transform()?,
            params: r.vec3()?,
            background_top: r.color()?,
            background_bottom: r.color()?,
        };
        // Tamaño mínimo de cada elemento, para rechazar recuentos imposibles.
        let lights = r.array(5, read_light)?;
        let textures = r.array(5, read_texture)?;
        let meshes = r.array(13, |r| read_mesh(r, version))?;
        let parts = r.array(54, |r| read_part(r, version))?;
        if r.pos != data.len() {
            return Err(E3dError::TrailingData(data.len() - r.pos));
        }
        let scene = Self {
            version,
            view,
            lights,
            textures,
            meshes,
            parts,
        };
        scene.validate()?;
        Ok(scene)
    }

    /// Índices de malla y de vértice dentro de rango.
    fn validate(&self) -> Result<(), E3dError> {
        for part in &self.parts {
            if usize::from(part.mesh) >= self.meshes.len() {
                return Err(E3dError::Invalid {
                    what: "meshId",
                    value: part.mesh.into(),
                    at: 0,
                });
            }
        }
        for mesh in &self.meshes {
            let n = mesh.vertex_count();
            let groups = mesh
                .faces
                .iter()
                .map(|f| &f.elements)
                .chain(mesh.edges.iter().map(|e| &e.elements));
            for el in groups {
                if let Some(&i) = el.indices.iter().find(|&&i| usize::from(i) >= n) {
                    return Err(E3dError::Invalid {
                        what: "índice de vértice",
                        value: i.into(),
                        at: 0,
                    });
                }
            }
        }
        Ok(())
    }

    pub fn mesh_of(&self, part: &Part) -> &Mesh {
        &self.meshes[usize::from(part.mesh)]
    }

    /// Caja envolvente en el mundo de las piezas que cumplen `filter`.
    pub fn bounds(&self, filter: impl Fn(&Part) -> bool) -> Option<Aabb> {
        let mut aabb: Option<Aabb> = None;
        for part in self.parts.iter().filter(|p| filter(p)) {
            if let Some(b) = self.part_bounds(part) {
                aabb = Some(aabb.map_or(b, |a| a.union(&b)));
            }
        }
        aabb
    }

    /// Caja envolvente de una pieza en el mundo.
    pub fn part_bounds(&self, part: &Part) -> Option<Aabb> {
        let mesh = self.mesh_of(part);
        let mut aabb: Option<Aabb> = None;
        for i in 0..mesh.vertex_count() {
            let p = transform_point(&part.transform, mesh.position(i));
            aabb = Some(match aabb {
                None => Aabb { min: p, max: p },
                Some(a) => a.extend(p),
            });
        }
        aabb
    }
}

/// Caja alineada con los ejes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn size(&self) -> Vec3 {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }

    fn extend(self, p: Vec3) -> Self {
        Self {
            min: std::array::from_fn(|k| self.min[k].min(p[k])),
            max: std::array::from_fn(|k| self.max[k].max(p[k])),
        }
    }

    fn union(self, o: &Self) -> Self {
        self.extend(o.min).extend(o.max)
    }
}

impl Mesh {
    pub fn vertex_count(&self) -> usize {
        if self.stride == 0 {
            0
        } else {
            self.vertices.len() / self.stride
        }
    }

    /// Posición del vértice `i` (origen si la malla no trae posiciones).
    pub fn position(&self, i: usize) -> Vec3 {
        if !self.has_positions {
            return [0.0; 3];
        }
        let o = i * self.stride;
        [self.vertices[o], self.vertices[o + 1], self.vertices[o + 2]]
    }

    /// Triángulos de todos los grupos de caras, como índices de vértice.
    /// Expande tiras y abanicos; descarta los degenerados por índice.
    pub fn triangles(&self) -> impl Iterator<Item = [u16; 3]> + '_ {
        self.faces
            .iter()
            .flat_map(|f| triangles_of(&f.elements))
            .filter(|t| t[0] != t[1] && t[1] != t[2] && t[0] != t[2])
    }

    /// Normales por vértice: las del fichero si las trae, o calculadas a
    /// partir de las caras (media ponderada por área) si no.
    pub fn normals(&self) -> Cow<'_, [Vec3]> {
        let n = self.vertex_count();
        if self.has_normals {
            let o = if self.has_positions { 3 } else { 0 };
            let stored = (0..n)
                .map(|i| {
                    let b = i * self.stride + o;
                    [self.vertices[b], self.vertices[b + 1], self.vertices[b + 2]]
                })
                .collect::<Vec<_>>();
            return Cow::Owned(stored);
        }
        let mut acc = vec![[0.0f32; 3]; n];
        // Respaldo para vértices cuyas caras se anulan (láminas de doble
        // cara que comparten vértices): la normal de la primera cara.
        let mut first = vec![[0.0f32; 3]; n];
        for t in self.triangles() {
            let [a, b, c] = t.map(|i| self.position(usize::from(i)));
            // Sin normalizar: la longitud es el doble del área.
            let face = cross(sub(b, a), sub(c, a));
            for i in t {
                let i = usize::from(i);
                for k in 0..3 {
                    acc[i][k] += face[k];
                }
                if first[i] == [0.0; 3] {
                    first[i] = normalize(face);
                }
            }
        }
        let normals = acc
            .into_iter()
            .zip(first)
            .map(|(v, f)| {
                let v = normalize(v);
                if v == [0.0; 3] { f } else { v }
            })
            .collect();
        Cow::Owned(normals)
    }
}

fn triangles_of(el: &Elements) -> Box<dyn Iterator<Item = [u16; 3]> + '_> {
    let ix = &el.indices;
    match el.mode {
        Primitive::Triangles => Box::new(ix.chunks_exact(3).map(|c| [c[0], c[1], c[2]])),
        Primitive::TriangleStrip => Box::new((2..ix.len()).map(move |i| {
            // Alterna el sentido para conservar la orientación.
            if i % 2 == 0 {
                [ix[i - 2], ix[i - 1], ix[i]]
            } else {
                [ix[i - 1], ix[i - 2], ix[i]]
            }
        })),
        Primitive::TriangleFan => Box::new((2..ix.len()).map(move |i| [ix[0], ix[i - 1], ix[i]])),
        _ => Box::new(std::iter::empty()),
    }
}

pub fn transform_point(m: &Mat4, p: Vec3) -> Vec3 {
    [
        m[0] * p[0] + m[4] * p[1] + m[8] * p[2] + m[12],
        m[1] * p[0] + m[5] * p[1] + m[9] * p[2] + m[13],
        m[2] * p[0] + m[6] * p[1] + m[10] * p[2] + m[14],
    ]
}

pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Vector unitario; el vector nulo se queda en cero.
pub fn normalize(v: Vec3) -> Vec3 {
    let len = dot(v, v).sqrt();
    if len > 0.0 && len.is_finite() {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0; 3]
    }
}

fn read_light(r: &mut Reader) -> Result<Light, E3dError> {
    let at = r.pos;
    let kind = match r.u8()? {
        0 => LightKind::Ambient,
        1 => LightKind::Directional,
        2 => LightKind::Point,
        3 => LightKind::Spot,
        v => {
            return Err(E3dError::Invalid {
                what: "tipo de luz",
                value: v.into(),
                at,
            });
        }
    };
    let color = r.color()?;
    let has_pos = matches!(kind, LightKind::Point | LightKind::Spot);
    let has_dir = matches!(kind, LightKind::Directional | LightKind::Spot);
    let position = if has_pos { Some(r.vec3()?) } else { None };
    let direction = if has_dir { Some(r.vec3()?) } else { None };
    let spot = if kind == LightKind::Spot {
        Some((r.f32()?, r.f32()?))
    } else {
        None
    };
    Ok(Light {
        kind,
        color,
        position,
        direction,
        spot,
    })
}

fn read_texture(r: &mut Reader) -> Result<Texture, E3dError> {
    r.u8()?; // formato de píxel: siempre BGRA en la práctica
    let width = r.u16()?;
    let height = r.u16()?;
    let raw = r.take(usize::from(width) * usize::from(height) * 4)?;
    let mut rgba = raw.to_vec();
    for px in rgba.chunks_exact_mut(4) {
        px.swap(0, 2);
    }
    Ok(Texture {
        width,
        height,
        rgba,
    })
}

fn read_mesh(r: &mut Reader, version: u8) -> Result<Mesh, E3dError> {
    let flags = r.u8()?;
    let has_positions = flags & 1 != 0;
    let has_normals = flags & 2 != 0;
    let has_tex_coords = flags & 4 != 0;
    let stride = 3 * usize::from(has_positions)
        + 3 * usize::from(has_normals)
        + 2 * usize::from(has_tex_coords);
    let at = r.pos;
    let vertices = r.f32_buffer()?;
    if stride > 0 && vertices.len() % stride != 0 {
        return Err(E3dError::Invalid {
            what: "longitud del buffer de vértices",
            value: vertices.len() as i64,
            at,
        });
    }
    let faces = r.array(12, |r| {
        Ok(FaceGroup {
            material: Material {
                color_valid: r.u8()? > 0,
                color: r.color()?,
                texture: u16::try_from(r.i16()?).ok(),
            },
            elements: read_elements(r, version)?,
        })
    })?;
    let edges = r.array(15, |r| {
        Ok(EdgeGroup {
            style: EdgeStyle {
                color: r.color()?,
                line_width: r.f32()?,
                line_type: r.u8()?,
            },
            elements: read_elements(r, version)?,
        })
    })?;
    Ok(Mesh {
        has_positions,
        has_normals,
        has_tex_coords,
        stride,
        vertices,
        faces,
        edges,
    })
}

fn read_elements(r: &mut Reader, version: u8) -> Result<Elements, E3dError> {
    let center = if version >= 2 { Some(r.vec3()?) } else { None };
    let at = r.pos;
    let mode = match r.u8()? {
        0 => Primitive::Points,
        1 => Primitive::LineStrip,
        2 => Primitive::LineLoop,
        3 => Primitive::Lines,
        4 => Primitive::TriangleStrip,
        5 => Primitive::TriangleFan,
        6 => Primitive::Triangles,
        v => {
            return Err(E3dError::Invalid {
                what: "modo de primitiva",
                value: v.into(),
                at,
            });
        }
    };
    let count = r.i32()?;
    let at = r.pos;
    let indices = match r.u8()? {
        0 => r.byte_buffer()?.iter().map(|&b| u16::from(b)).collect(),
        1 => r.u16_buffer()?,
        v => {
            return Err(E3dError::Invalid {
                what: "tipo de índice",
                value: v.into(),
                at,
            });
        }
    };
    Ok(Elements {
        center,
        mode,
        count,
        indices,
    })
}

fn read_part(r: &mut Reader, version: u8) -> Result<Part, E3dError> {
    let at = r.pos;
    let mesh = u16::try_from(r.i16()?).map_err(|_| E3dError::Invalid {
        what: "meshId",
        value: -1,
        at,
    })?;
    let transform = r.transform()?;
    let color = r.color()?;
    let (type_id, object_id) = if version >= 3 {
        (Some(r.i16()? as u16), Some(r.i32()?))
    } else {
        (None, None)
    };
    let texts = if version >= 4 {
        r.array(58, |r| {
            Ok(TextLine {
                transform: r.transform()?,
                height: r.f32()?,
                justification: r.i16()?,
                text: r.string()?,
            })
        })?
    } else {
        Vec::new()
    };
    Ok(Part {
        mesh,
        transform,
        color,
        type_id,
        object_id,
        texts,
    })
}

/// Lector secuencial little-endian con comprobación de límites.
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
    limits: Limits,
}

impl<'a> Reader<'a> {
    fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], E3dError> {
        if n > self.remaining() {
            return Err(E3dError::UnexpectedEof {
                at: self.pos,
                needed: n,
            });
        }
        let s = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    fn bytes<const N: usize>(&mut self) -> Result<[u8; N], E3dError> {
        let s = self.take(N)?;
        Ok(s.try_into().expect("longitud comprobada"))
    }

    fn u8(&mut self) -> Result<u8, E3dError> {
        Ok(self.bytes::<1>()?[0])
    }

    fn u16(&mut self) -> Result<u16, E3dError> {
        Ok(u16::from_le_bytes(self.bytes()?))
    }

    fn i16(&mut self) -> Result<i16, E3dError> {
        Ok(i16::from_le_bytes(self.bytes()?))
    }

    fn i32(&mut self) -> Result<i32, E3dError> {
        Ok(i32::from_le_bytes(self.bytes()?))
    }

    fn f32(&mut self) -> Result<f32, E3dError> {
        Ok(f32::from_le_bytes(self.bytes()?))
    }

    fn vec3(&mut self) -> Result<Vec3, E3dError> {
        Ok([self.f32()?, self.f32()?, self.f32()?])
    }

    /// RGBA en bytes; el alfa del fichero es transparencia.
    fn color(&mut self) -> Result<Rgba, E3dError> {
        let [r, g, b, a] = self.bytes::<4>()?;
        Ok([
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
            1.0 - f32::from(a) / 255.0,
        ])
    }

    /// Cuatro vectores fila (ejes x, y, z y traslación) → column-major.
    fn transform(&mut self) -> Result<Mat4, E3dError> {
        let [x, y, z, t] = [self.vec3()?, self.vec3()?, self.vec3()?, self.vec3()?];
        Ok([
            x[0], x[1], x[2], 0.0, y[0], y[1], y[2], 0.0, z[0], z[1], z[2], 0.0, t[0], t[1], t[2],
            1.0,
        ])
    }

    /// Longitud en bytes (`long`) validada contra lo que queda.
    fn byte_len(&mut self) -> Result<usize, E3dError> {
        let at = self.pos;
        let n = self.i32()?;
        let n = usize::try_from(n).map_err(|_| E3dError::Invalid {
            what: "longitud",
            value: n.into(),
            at,
        })?;
        if n > self.remaining() {
            return Err(E3dError::UnexpectedEof { at, needed: n });
        }
        Ok(n)
    }

    fn byte_buffer(&mut self) -> Result<&'a [u8], E3dError> {
        let n = self.byte_len()?;
        self.take(n)
    }

    fn u16_buffer(&mut self) -> Result<Vec<u16>, E3dError> {
        let b = self.byte_buffer()?;
        Ok(b.chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect())
    }

    fn f32_buffer(&mut self) -> Result<Vec<f32>, E3dError> {
        let b = self.byte_buffer()?;
        Ok(b.chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect())
    }

    /// UTF-16LE con prefijo de longitud en bytes; sin terminadores nulos.
    fn string(&mut self) -> Result<String, E3dError> {
        let b = self.byte_buffer()?;
        let units = b
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .filter(|&u| u != 0);
        Ok(char::decode_utf16(units)
            .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect())
    }

    /// Array con prefijo de número de elementos. `min_size` es lo mínimo que
    /// ocupa cada elemento: un recuento que no cabe en lo que queda del
    /// fichero se rechaza antes de reservar memoria.
    fn array<T>(
        &mut self,
        min_size: usize,
        mut item: impl FnMut(&mut Self) -> Result<T, E3dError>,
    ) -> Result<Vec<T>, E3dError> {
        let at = self.pos;
        let n = self.i32()?;
        let invalid = E3dError::Invalid {
            what: "número de elementos",
            value: n.into(),
            at,
        };
        let n = usize::try_from(n).map_err(|_| invalid.clone())?;
        if n > self.limits.max_array_len || n.saturating_mul(min_size) > self.remaining() {
            return Err(invalid);
        }
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            out.push(item(self)?);
        }
        Ok(out)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// E3D mínimo v4: un triángulo sin normales en una pieza trasladada.
    pub(crate) fn tiny() -> Vec<u8> {
        let mut b = vec![4u8];
        let f = |b: &mut Vec<u8>, v: f32| b.extend_from_slice(&v.to_le_bytes());
        let i = |b: &mut Vec<u8>, v: i32| b.extend_from_slice(&v.to_le_bytes());
        let ident = |b: &mut Vec<u8>, t: [f32; 3]| {
            for row in [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.], t] {
                for v in row {
                    b.extend_from_slice(&f32::to_le_bytes(v));
                }
            }
        };
        ident(&mut b, [0.; 3]);
        for _ in 0..3 {
            f(&mut b, 0.0);
        }
        b.extend_from_slice(&[0; 8]); // fondos
        i(&mut b, 0); // luces
        i(&mut b, 0); // texturas
        i(&mut b, 1); // mallas
        b.push(1); // solo posiciones
        i(&mut b, 36);
        for v in [0., 0., 0., 10., 0., 0., 0., 20., 0.] {
            f(&mut b, v);
        }
        i(&mut b, 1); // caras
        b.push(1);
        b.extend_from_slice(&[255, 0, 0, 0]);
        b.extend_from_slice(&(-1i16).to_le_bytes());
        for _ in 0..3 {
            f(&mut b, 0.0);
        }
        b.push(6);
        i(&mut b, 3);
        b.push(1);
        i(&mut b, 6);
        for v in [0u16, 1, 2] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        i(&mut b, 0); // aristas
        i(&mut b, 1); // piezas
        b.extend_from_slice(&0i16.to_le_bytes());
        ident(&mut b, [100., 0., 5.]);
        b.extend_from_slice(&[0, 0, 255, 128]);
        b.extend_from_slice(&180i16.to_le_bytes());
        i(&mut b, 42);
        i(&mut b, 0); // textos
        b
    }

    #[test]
    fn parses_minimal_file() {
        let s = Scene::parse(&tiny()).unwrap();
        assert_eq!(s.version, 4);
        assert_eq!(s.parts[0].key().as_deref(), Some("180/42"));
        assert_eq!(s.parts[0].kind(), PartKind::Device);
        let b = s.bounds(|_| true).unwrap();
        assert_eq!(b.min, [100., 0., 5.]);
        assert_eq!(b.max, [110., 20., 5.]);
        let n = s.meshes[0].normals();
        assert_eq!(n[0], [0., 0., 1.]);
    }

    #[test]
    fn rejects_truncated_and_huge_counts() {
        let data = tiny();
        for cut in [1, 10, 100, data.len() - 1] {
            assert!(Scene::parse(&data[..cut]).is_err(), "cortado en {cut}");
        }
        // Recuento de luces enorme: se rechaza sin reservar memoria.
        let mut bad = data.clone();
        let at = 1 + 64 + 12 + 8;
        bad[at..at + 4].copy_from_slice(&i32::MAX.to_le_bytes());
        assert!(matches!(Scene::parse(&bad), Err(E3dError::Invalid { .. })));
        let mut v5 = data;
        v5[0] = 5;
        assert_eq!(
            Scene::parse(&v5).unwrap_err(),
            E3dError::UnsupportedVersion(5)
        );
    }
}
