//! Comprueba que el `.epdz` de ejemplo se abre con `sevenz-rust2` y que sus
//! páginas SVG se pueden sanear.

use std::path::PathBuf;
use std::time::Instant;

use ecad_core::{Archive, svg};

fn sample() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../samples/ejemplo.epdz")
}

#[test]
fn opens_sample_epdz() {
    let t = Instant::now();
    let archive = Archive::open(sample()).expect("abrir ejemplo.epdz");
    eprintln!("{} entradas en {:?}", archive.entries().len(), t.elapsed());

    let db = archive.get("manifest.db").expect("manifest.db");
    assert!(db.starts_with(b"SQLite format 3\0"));

    let pages: Vec<_> = archive.with_extension("svg").collect();
    assert!(!pages.is_empty(), "sin páginas SVG");
    assert!(archive.with_extension("e3d").next().is_some(), "sin E3D");
    assert!(archive.with_extension("aml").next().is_some(), "sin AML");

    let t = Instant::now();
    for page in &pages {
        let text = std::str::from_utf8(&page.data).expect("SVG UTF-8");
        let clean = svg::sanitize(text).unwrap_or_else(|e| panic!("{}: {e}", page.name));
        assert!(!clean.contains("<script"), "{}", page.name);
        assert!(!clean.contains("javascript:"), "{}", page.name);
        assert!(svg::title(text).is_some(), "{} sin <title>", page.name);
    }
    eprintln!("{} páginas saneadas en {:?}", pages.len(), t.elapsed());
}

#[test]
fn manifest_db_opens_with_rusqlite() {
    let archive = Archive::open(sample()).expect("abrir ejemplo.epdz");
    let path = std::env::temp_dir().join(format!("ecad-core-test-{}.db", std::process::id()));
    std::fs::write(&path, archive.get("manifest.db").unwrap()).unwrap();
    let db =
        rusqlite::Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("abrir manifest.db");
    let pages: i64 = db
        .query_row(
            "SELECT count(*) FROM package WHERE type = 'page'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    drop(db);
    let _ = std::fs::remove_file(&path);
    assert_eq!(pages as usize, archive.with_extension("svg").count());
}
