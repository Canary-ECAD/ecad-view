//! STL binario a partir de piezas de un E3D.
//!
//! Cada pieza se lleva a coordenadas del mundo con su transformación; las
//! coordenadas quedan en milímetros con Z hacia arriba (la convención del
//! propio E3D). Las normales de faceta se recalculan siempre a partir de los
//! vértices ya transformados; los triángulos de área nula se descartan.

use crate::e3d::{Mat4, Part, Scene, Vec3, cross, normalize, sub, transform_point};

/// Cabecera de 80 bytes. No empieza por `solid`, que algunos lectores toman
/// por STL ASCII.
const HEADER: &[u8] = b"ecad-view STL binario, mm, Z arriba";

/// STL binario con las piezas dadas. Devuelve el fichero y el número de
/// triángulos.
pub fn to_stl<'a>(scene: &Scene, parts: impl IntoIterator<Item = &'a Part>) -> (Vec<u8>, u32) {
    let mut body = Vec::new();
    let mut count: u32 = 0;
    for part in parts {
        let mesh = scene.mesh_of(part);
        if !mesh.has_positions {
            continue;
        }
        // Una transformación con simetría invierte el sentido de giro: se
        // reordenan los vértices para que la normal siga hacia fuera.
        let mirrored = det3(&part.transform) < 0.0;
        for t in mesh.triangles() {
            let [a, b, c] =
                t.map(|i| transform_point(&part.transform, mesh.position(usize::from(i))));
            let (b, c) = if mirrored { (c, b) } else { (b, c) };
            let n = normalize(cross(sub(b, a), sub(c, a)));
            if n == [0.0; 3] {
                continue;
            }
            for v in [n, a, b, c] {
                push_vec(&mut body, v);
            }
            body.extend_from_slice(&[0, 0]);
            count += 1;
        }
    }
    let mut out = Vec::with_capacity(84 + body.len());
    out.extend_from_slice(HEADER);
    out.resize(80, b' ');
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&body);
    (out, count)
}

fn push_vec(out: &mut Vec<u8>, v: Vec3) {
    for x in v {
        out.extend_from_slice(&x.to_le_bytes());
    }
}

/// Determinante de la parte 3x3 (column-major).
fn det3(m: &Mat4) -> f32 {
    let x = [m[0], m[1], m[2]];
    let y = [m[4], m[5], m[6]];
    let z = [m[8], m[9], m[10]];
    crate::e3d::dot(x, cross(y, z))
}

/// Triángulos de un STL binario (normal y tres vértices). Para tests y
/// herramientas: comprueba que el tamaño cuadra con el recuento.
pub fn read_stl(data: &[u8]) -> Option<Vec<[Vec3; 4]>> {
    let count = u32::from_le_bytes(data.get(80..84)?.try_into().ok()?) as usize;
    if data.len() != 84 + count * 50 {
        return None;
    }
    let f = |b: &[u8]| f32::from_le_bytes([b[0], b[1], b[2], b[3]]);
    let tris = data[84..]
        .chunks_exact(50)
        .map(|c| std::array::from_fn(|k| std::array::from_fn(|j| f(&c[k * 12 + j * 4..]))))
        .collect();
    Some(tris)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_mirror() {
        let scene = crate::e3d::Scene::parse(&crate::e3d::tests::tiny()).unwrap();
        let (stl, n) = to_stl(&scene, &scene.parts);
        assert_eq!(n, 1);
        let tris = read_stl(&stl).unwrap();
        assert_eq!(tris[0][0], [0.0, 0.0, 1.0]);
        assert_eq!(tris[0][1], [100.0, 0.0, 5.0]);

        let mut mirrored = scene.parts[0].clone();
        mirrored.transform[0] = -1.0;
        let (stl, _) = to_stl(&scene, [&mirrored]);
        // Sigue mirando hacia +Z aunque la pieza esté reflejada en X.
        assert_eq!(read_stl(&stl).unwrap()[0][0], [0.0, 0.0, 1.0]);
    }
}
