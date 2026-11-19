//! E3D de `samples/ejemplo.epdz` (criterios del sprint 2).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use ecad_core::Epdz;
use ecad_core::e3d::{PartKind, Primitive, Scene};

fn sample() -> &'static Epdz {
    static SAMPLE: OnceLock<Epdz> = OnceLock::new();
    SAMPLE.get_or_init(|| {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../samples/ejemplo.epdz");
        Epdz::open(path).expect("abrir ejemplo.epdz")
    })
}

fn scenes() -> Vec<(String, Scene)> {
    sample()
        .models()
        .map(|e| {
            let name = e.name.rsplit('/').next().unwrap().to_owned();
            let scene = Scene::parse(&e.data).unwrap_or_else(|err| panic!("{name}: {err}"));
            (name, scene)
        })
        .collect()
}

fn round(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// Mismo resumen que calcula `parseE3d` de la v1 (`packages/e3d-core`):
/// versión, luces, texturas, mallas, piezas, floats de vértices, índices,
/// suma de vértices, suma de transformaciones e ids, líneas de texto y
/// caracteres. Los valores esperados salen de ejecutar la v1 sobre el
/// mismo archivo.
fn digest(name: &str, s: &Scene) -> String {
    let (mut verts, mut idx, mut vsum) = (0usize, 0usize, 0f64);
    for m in &s.meshes {
        verts += m.vertices.len();
        vsum += m.vertices.iter().map(|&v| f64::from(v)).sum::<f64>();
        let groups = m.faces.iter().map(|f| &f.elements);
        let groups = groups.chain(m.edges.iter().map(|e| &e.elements));
        idx += groups.map(|e| e.indices.len()).sum::<usize>();
    }
    let (mut tsum, mut texts, mut chars) = (0f64, 0usize, 0usize);
    for p in &s.parts {
        for &v in &p.transform {
            tsum += f64::from(v);
        }
        tsum += f64::from(p.type_id.unwrap_or(0))
            + f64::from(p.object_id.unwrap_or(0))
            + f64::from(p.mesh);
        texts += p.texts.len();
        // La v1 cuenta unidades UTF-16.
        chars += p
            .texts
            .iter()
            .map(|t| t.text.encode_utf16().count())
            .sum::<usize>();
    }
    format!(
        "{name} {} {} {} {} {} {verts} {idx} {} {} {texts} {chars}",
        s.version,
        s.lights.len(),
        s.textures.len(),
        s.meshes.len(),
        s.parts.len(),
        round(vsum),
        round(tsum),
    )
}

#[test]
fn matches_v1_reader() {
    let expected = "\
38195.E3d 4 2 0 329 696 646917 952572 62733421.184 38681908.618 145 907
38404.E3d 4 2 0 488 1162 1186482 1557331 153466752.174 64063752.281 320 2147
38796.E3d 4 2 0 67 241 76434 164566 14629491.902 15053952.924 23 136
38845.E3d 4 2 0 64 211 120447 259088 23072033.152 12771115.589 46 298
38917.E3d 4 2 0 56 177 67545 140344 11035051.552 10812198.641 23 136
43966.E3d 4 2 0 61 192 160836 431462 50670236.603 10594957.469 17 58
58001.E3d 4 2 0 149 453 342951 730370 14128615.8 27519410.282 91 294";
    let mut got: Vec<String> = scenes().iter().map(|(n, s)| digest(n, s)).collect();
    got.sort();
    assert_eq!(got.join("\n"), expected);
}

#[test]
fn observed_semantics_hold() {
    let mut kinds: BTreeMap<u16, usize> = BTreeMap::new();
    let mut without_normals = 0;
    let mut meshes = 0;
    for (_, s) in scenes() {
        assert_eq!(s.lights.len(), 2);
        for m in &s.meshes {
            meshes += 1;
            without_normals += usize::from(!m.has_normals);
            assert!(
                m.faces
                    .iter()
                    .all(|f| f.elements.mode == Primitive::Triangles)
            );
            assert!(m.edges.iter().all(|e| e.elements.mode == Primitive::Lines));
            // Normales calculadas o leídas: unitarias en todos los vértices
            // que tocan alguna cara.
            let n = m.normals();
            for t in m.triangles() {
                for i in t {
                    let v = n[usize::from(i)];
                    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
                    assert!((len - 1.0).abs() < 1e-3, "normal {v:?}");
                }
            }
        }
        for p in &s.parts {
            *kinds.entry(p.type_id.unwrap()).or_default() += 1;
        }
    }
    assert_eq!(meshes, 1214);
    assert_eq!(without_normals, 617);
    assert_eq!(kinds[&166], 580);
    assert_eq!(kinds[&180], 1151);
    assert_eq!(kinds[&175], 1237);
}

#[test]
fn kinds_and_physical_filter() {
    for (_, s) in scenes() {
        for p in &s.parts {
            let k = p.kind();
            assert_eq!(
                k.is_physical(),
                !matches!(
                    k,
                    PartKind::RestrictedArea
                        | PartKind::Cutout
                        | PartKind::MountingSurface
                        | PartKind::Unknown
                )
            );
        }
    }
}
