//! AutomationML de `samples/ejemplo.epdz` (criterios del sprint 2).

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use ecad_core::e3d::{PartKind, Scene};
use ecad_core::{Aml, Epdz, Project};

struct Sample {
    epdz: Epdz,
    project: Project,
    aml: Aml,
    parse_time: Duration,
}

fn sample() -> &'static Sample {
    static SAMPLE: OnceLock<Sample> = OnceLock::new();
    SAMPLE.get_or_init(|| {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../samples/ejemplo.epdz");
        let epdz = Epdz::open(path).expect("abrir ejemplo.epdz");
        let project = epdz.project();
        let t = Instant::now();
        let aml = epdz.aml().expect("leer AML").expect("el ejemplo trae AML");
        let parse_time = t.elapsed();
        eprintln!("AML leído en {parse_time:?}");
        Sample {
            epdz,
            project,
            aml,
            parse_time,
        }
    })
}

fn count(f: impl Fn(&ecad_core::aml::Element) -> bool) -> usize {
    sample().aml.elements.iter().filter(|e| f(e)).count()
}

#[test]
fn fast_enough() {
    // Criterio: < 5 s. En debug el margen es menor; en release tarda
    // décimas de segundo.
    assert!(sample().parse_time < Duration::from_secs(5));
}

/// Mismos recuentos que `parseAml` de la v1 (`packages/e3d-core/src/aml.ts`)
/// sobre el mismo fichero.
#[test]
fn matches_v1_reader() {
    let aml = &sample().aml;
    // La v1 cuenta también los 12 `InternalElement` de dentro de los
    // artículos (3318); la v2 solo los de la jerarquía de instancias.
    assert_eq!(aml.elements.len(), 3306);
    assert_eq!(count(|e| e.part_number.is_some()), 1005);
    assert_eq!(count(|e| e.oid.is_some()), 3295);
    assert_eq!(count(|e| e.frame.is_some()), 3306 - 12);
    assert_eq!(count(|e| e.surface.is_some()), 512);
    assert_eq!(count(|e| e.full_designation.is_some()), 512);
    assert_eq!(count(|e| e.layout_space.is_some()), 7);
    assert_eq!(count(|e| e.function_text.is_some()), 609);
    assert_eq!(count(|e| e.role.is_some()), 3295);
    assert_eq!(
        count(|e| e.eclass.as_ref().is_some_and(|c| c.code.is_some())),
        1232
    );
    assert_eq!(count(|e| e.interfaces.len() > 0), 9);
    assert_eq!(count(|e| e.dimensions.diameter.is_some()), 284);
    // La v1 no lee `Nominal diameter` (20 taladros roscados: `M5`…).
    assert_eq!(count(|e| e.dimensions.thread.is_some()), 20);
    assert!(aml.languages.len() >= 19);
    assert_eq!(aml.project_name.as_deref(), Some("Sample_Project"));
    let desc = aml.project_description.as_ref().unwrap();
    assert_eq!(desc["en-US"], "Grinding machine");
    assert_eq!(aml.bridge_groups.len(), 1);
    assert_eq!(aml.bridge_groups[0].kind, "Plug-in bridge");
    assert_eq!(aml.bridge_groups[0].object_ids.len(), 3);
    assert_eq!(aml.writer.version.as_deref(), Some("2025.0.3.23756"));
}

#[test]
fn articles() {
    let aml = &sample().aml;
    assert_eq!(aml.articles.len(), 148);
    let with_article = count(|e| e.article.is_some());
    assert_eq!(with_article, 1003);
    // Toda referencia a artículo resuelve.
    for e in aml.elements.iter().filter(|e| e.article.is_some()) {
        assert!(aml.article_of(e).is_some(), "{}: sin artículo", e.name);
    }
    let a = &aml.articles["2500210"];
    assert_eq!(a.manufacturer.as_deref(), Some("Rittal"));
    assert_eq!(a.product_type.as_deref(), Some("SZ.2500210"));
    assert_eq!(a.weight, Some(0.614));
    assert_eq!(a.dimensions.width, Some(437.0));
    assert!(
        a.links
            .iter()
            .any(|l| l.url == "http://www.rittal.com/es-es/part/2500210")
    );
    assert_eq!(
        aml.articles.values().filter(|a| a.gtin.is_some()).count(),
        30
    );
    assert_eq!(
        aml.articles.values().filter(|a| a.weight.is_some()).count(),
        86
    );

    let a = &aml.articles["9340030"];
    let mp = &a.mounting.fixing_variants[0].mounting_points[0];
    assert_eq!(mp.position.unwrap().position, [10.0, 12.0, 0.0]);
    assert_eq!(mp.fixings[0].execution.as_deref(), Some("Drill hole"));
    assert_eq!(mp.fixings[0].hole_diameter, Some(5.5));
    let holes: usize = aml
        .articles
        .values()
        .flat_map(|a| &a.mounting.fixing_variants)
        .flat_map(|v| &v.mounting_points)
        .flat_map(|p| &p.fixings)
        .filter(|f| f.hole_diameter.is_some())
        .count();
    assert_eq!(holes, 25);
}

#[test]
fn placement_and_surfaces() {
    let aml = &sample().aml;
    let ea1 = aml
        .elements
        .iter()
        .position(|e| e.name == "=E01+A1-EA1")
        .unwrap();
    let e = &aml.elements[ea1];
    let f = e.frame.unwrap();
    assert_eq!((f.x, f.y, f.z, f.rz), (130.85, 36.5, -2.0, -90.0));
    let surface = &aml.elements[e.mounted_on.unwrap()];
    assert_eq!(surface.name, "+A1-U3*9");
    let s = surface.surface.unwrap();
    assert_eq!(s.size, [711.0, 23.5]);
    assert_eq!(s.normal, [0.0, 1.0, 0.0]);
    assert_eq!(
        surface.full_designation.as_deref(),
        Some("S1:Nivel de perfiles frontal interior")
    );
    assert_eq!(
        aml.elements[e.space.unwrap()].layout_space.as_deref(),
        Some("A1")
    );

    // Todas las piezas con artículo están en un espacio de montaje, y casi
    // todas sobre una superficie. Las que no: el armario, sus accesorios
    // colgados de un perfil (zócalos, paneles, pasacables) y las barras de
    // los haces de cobre.
    for e in aml.elements.iter().filter(|e| e.part_number.is_some()) {
        assert!(e.space.is_some(), "{} sin espacio", e.name);
        if e.mounted_on.is_none() {
            let role = e.role.as_deref().unwrap_or_default();
            assert!(
                [
                    "Enclosure",
                    "Base",
                    "Base element",
                    "Socket orifice",
                    "Panel",
                    "Busbar"
                ]
                .contains(&role),
                "{} ({role}) sin superficie",
                e.name
            );
        }
    }

    // Placa de una pieza: el elemento que lleva la superficie.
    let rail = aml
        .elements
        .iter()
        .position(|e| e.name == "+A1-U18")
        .unwrap();
    let plate = aml.plate_of(rail).unwrap();
    assert_eq!(aml.elements[plate].role.as_deref(), Some("Mounting panel"));
    assert_eq!(aml.elements[plate].name, "+A1-U3*10");

    // Taladros con diámetro.
    let holes =
        count(|e| e.role.as_deref() == Some("Drill hole") && e.dimensions.diameter.is_some());
    assert_eq!(holes, 284);
}

/// Criterio: cada pieza con artículo queda enlazada a su función del manifest.
#[test]
fn every_article_links_to_a_manifest_function() {
    let s = sample();
    let links = s.aml.link_functions(&s.project);
    let functions: HashMap<i64, &ecad_core::project::Function> =
        s.project.functions.iter().map(|f| (f.id, f)).collect();
    for (i, e) in s.aml.elements.iter().enumerate() {
        if e.part_number.is_some() {
            let id = links
                .get(&i)
                .unwrap_or_else(|| panic!("{} sin función", e.name));
            assert_eq!(functions[id].oid, e.oid);
        }
    }
    assert!(links.len() >= 1005);
}

/// Identificación de los `typeId` del E3D cruzándolos con el AML.
#[test]
fn e3d_type_ids_against_aml() {
    let s = sample();
    let by_key = s.aml.by_key();
    let mut roles: BTreeMap<(u16, String), usize> = BTreeMap::new();
    let mut missing: BTreeMap<u16, usize> = BTreeMap::new();
    for entry in s.epdz.models() {
        let scene = Scene::parse(&entry.data).unwrap();
        for p in &scene.parts {
            let t = p.type_id.unwrap();
            match p.key().and_then(|k| by_key.get(k.as_str()).copied()) {
                Some(i) => {
                    let role = s.aml.elements[i].role.clone().unwrap_or_default();
                    *roles.entry((t, role)).or_default() += 1;
                }
                None => *missing.entry(t).or_default() += 1,
            }
        }
    }
    let role_of = |t: u16| -> Vec<&str> {
        roles
            .keys()
            .filter(|(id, _)| *id == t)
            .map(|(_, r)| r.as_str())
            .collect()
    };
    assert_eq!(
        role_of(175),
        ["Restricted cut-out area", "Restricted placing area"]
    );
    assert!(
        role_of(176)
            .iter()
            .all(|r| ["Drill hole", "Threaded hole", "Rectangle"].contains(r))
    );
    assert_eq!(role_of(173), ["Wire duct"]);
    assert_eq!(role_of(178), ["Mounting rail"]);
    assert_eq!(role_of(174), ["Busbar"]);
    // Los hilos enrutados no están en el AML; 185, 187 y 238 tampoco.
    for t in [166, 185, 187, 238] {
        assert!(role_of(t).is_empty(), "{t}");
    }
    assert_eq!(missing[&166], 580);
    assert_eq!(PartKind::from_type_id(Some(175)), PartKind::RestrictedArea);
    eprintln!("{roles:#?}\nsin AML: {missing:?}");
}
