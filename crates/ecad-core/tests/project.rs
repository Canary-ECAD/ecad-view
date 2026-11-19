//! Modelo de proyecto leído de `samples/ejemplo.epdz` (criterios del sprint 1).

use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Instant;

use ecad_core::project::{Certainty, ConnectionKind, FunctionKind};
use ecad_core::{Epdz, Project};

fn sample() -> &'static (Epdz, Project) {
    static SAMPLE: OnceLock<(Epdz, Project)> = OnceLock::new();
    SAMPLE.get_or_init(|| {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../samples/ejemplo.epdz");
        let t = Instant::now();
        let epdz = Epdz::open(path).expect("abrir ejemplo.epdz");
        let project = epdz.project();
        eprintln!("proyecto leído en {:?}", t.elapsed());
        (epdz, project)
    })
}

fn project() -> &'static Project {
    &sample().1
}

#[test]
fn counts_match_the_sample() {
    let p = project();
    assert_eq!(p.connections.len(), 847);
    assert_eq!(p.pages.len(), 10);
    assert_eq!(p.installation_spaces.len(), 7);
    assert_eq!(p.functions.len(), 3186);
    assert_eq!(p.interruption_points.len(), 66);
    assert_eq!(p.structure.len(), 67);
    assert_eq!(p.counts["mergedconnection"], 847);
}

#[test]
fn header() {
    let h = &project().header;
    assert_eq!(h.name.as_deref(), Some("Sample_Project"));
    assert_eq!(h.description.as_deref(), Some("Rectificadora"));
    assert_eq!(h.language.as_deref(), Some("es_ES"));
    assert_eq!(h.eplan_version.as_deref(), Some("2025.0.3.23756"));
    assert_eq!(h.company.as_deref(), Some("Muster GmbH"));
}

#[test]
fn accents_are_decoded() {
    let p = project();
    assert_eq!(
        p.pages[0].description.as_deref(),
        Some("Alimentación 400/230 V")
    );
    let cross_sections: Vec<_> = p
        .connections
        .iter()
        .filter_map(|c| c.cross_section.as_deref())
        .collect();
    assert!(cross_sections.contains(&"0,75 mm²"), "{cross_sections:?}");
    assert!(
        p.structure
            .iter()
            .any(|s| s.description.as_deref() == Some("Estación \"Suministrar\""))
    );
    let json = serde_json::to_string(p).unwrap();
    assert!(
        !json.contains('\u{FFFD}'),
        "carácter de reemplazo en el JSON"
    );
}

#[test]
fn pages_in_project_order_with_their_svg() {
    let (epdz, p) = sample();
    let designations: Vec<_> = p
        .pages
        .iter()
        .map(|x| x.designation.full.as_str())
        .collect();
    assert_eq!(designations[..2], ["=GB1+A1&EFS1/1", "=GB1+A1&EFS1/2"]);
    assert_eq!(designations[9], "=MA3+A1&EFS1/1");
    for page in &p.pages {
        let file = page.file.as_deref().expect("página sin SVG");
        assert!(epdz.get(file).is_some(), "{file} no está en el archivo");
        assert_eq!(page.designation.document_type.as_deref(), Some("EFS1"));
    }
    let functions: usize = p.pages.iter().map(|x| x.functions.len()).sum();
    assert_eq!(functions, 608);
}

#[test]
fn installation_spaces_have_their_model() {
    let (epdz, p) = sample();
    for space in &p.installation_spaces {
        let file = space.file.as_deref().expect("espacio sin E3D");
        assert!(epdz.get(file).is_some(), "{file} no está en el archivo");
    }
    assert_eq!(p.installation_spaces[0].label.as_deref(), Some("+A1 (A1)"));
    assert_eq!(epdz.models().count(), 7);
}

#[test]
fn every_connection_has_source_and_target() {
    for c in &project().connections {
        assert!(c.source.designation.device.is_some(), "{c:?}");
        assert!(c.target.designation.device.is_some(), "{c:?}");
        assert!(c.svg_id.is_some(), "{}", c.name);
    }
}

#[test]
fn connections_are_classified() {
    let p = project();
    let n = |kind| p.connections.iter().filter(|c| c.kind == kind).count();
    assert_eq!(n(ConnectionKind::Tube), 25);
    assert_eq!(n(ConnectionKind::CableCore), 209);
    assert!(n(ConnectionKind::Bridge) > 0);
    assert_eq!(
        n(ConnectionKind::Wire) + n(ConnectionKind::Bridge),
        847 - 25 - 209
    );
    for c in &p.connections {
        if c.kind == ConnectionKind::CableCore {
            assert!(c.cable.as_deref().is_some_and(|x| x.contains("-WD")));
        }
        if c.kind == ConnectionKind::Bridge {
            assert!(c.source.designation.same_device(&c.target.designation));
        }
    }
}

#[test]
fn end_data() {
    let p = project();
    let with_treatment = p
        .connections
        .iter()
        .filter(|c| c.source.treatment.is_some())
        .count();
    assert_eq!(with_treatment, 732);
    assert_eq!(
        p.connections
            .iter()
            .filter(|c| c.daisy_chain.is_some())
            .count(),
        121
    );
    // La dirección de salida se asigna a su extremo aunque EPLAN la dé cruzada.
    let crossed = p
        .connections
        .iter()
        .find(|c| c.source.designation.full == "=GB1+A1-PF3:X2")
        .unwrap();
    assert_eq!(crossed.source.exit_direction.as_ref().unwrap().value, "TL");
    assert_eq!(crossed.target.exit_direction.as_ref().unwrap().value, "TR");
    let directions = p
        .connections
        .iter()
        .filter(|c| c.source.exit_direction.is_some() && c.target.exit_direction.is_some())
        .count();
    assert_eq!(directions, 545);
}

#[test]
fn nothing_unconfirmed_is_presented_as_certain() {
    for c in &project().connections {
        for end in [&c.source, &c.target] {
            for v in [&end.exit_direction, &end.torque, &end.tool_size]
                .into_iter()
                .flatten()
            {
                assert_eq!(v.certainty, Certainty::Unconfirmed);
            }
            for v in end.strip_length.iter() {
                assert_eq!(v.certainty, Certainty::Probable);
            }
        }
        for key in c.unidentified.keys() {
            assert!(
                [
                    "ep.31006", "ep.31058", "ep.31092", "ep.31114", "ep.31115", "ep.31155"
                ]
                .contains(&key.as_str())
                    || key.ends_with(".35573")
                    || key.ends_with(".35578"),
                "{key}"
            );
        }
    }
}

#[test]
fn json_has_every_connection_field() {
    let json = serde_json::to_value(&project().connections[1]).unwrap();
    for key in [
        "kind",
        "source",
        "target",
        "color",
        "cross_section",
        "length",
        "conductor_type",
        "potential",
        "signal",
        "cable",
        "daisy_chain",
        "wire_3d",
        "connection_oids",
        "route",
        "designation",
        "unidentified",
    ] {
        assert!(json.get(key).is_some(), "falta {key}");
    }
    for key in [
        "designation",
        "treatment",
        "strip_length",
        "exit_direction",
        "position",
        "torque",
        "tool_size",
    ] {
        assert!(json["source"].get(key).is_some(), "falta source.{key}");
    }
    assert_eq!(json["source"]["exit_direction"]["certainty"], "unconfirmed");
    assert_eq!(json["route"]["certainty"], "probable");
}

#[test]
fn functions() {
    let p = project();
    let terminal = p
        .functions
        .iter()
        .find(|f| f.designation.full == "+A1-XD5:24" && f.kind == FunctionKind::Terminal)
        .expect("borne +A1-XD5:24");
    assert_eq!(terminal.device.as_deref(), Some("+A1-XD5"));
    assert_eq!(terminal.parts[0].number, "PXC.3211813");
    assert_eq!(
        terminal.parts[0].description.as_deref(),
        Some("Borne de paso")
    );
    assert!(p.functions.iter().any(|f| f.kind == FunctionKind::Device));
    let on_pages = p.functions.iter().filter(|f| !f.pages.is_empty()).count();
    assert!(on_pages > 0);
}
