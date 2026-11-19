//! `ecad`: herramientas de línea de comandos sobre exportaciones de EPLAN.

use std::collections::BTreeMap;
use std::io::Write;
use std::process::ExitCode;

use ecad_core::project::{Connection, ConnectionKind, FunctionKind};
use ecad_core::{Archive, Epdz, Project};

const USAGE: &str = "\
Uso:
  ecad ls <archivo.epdz>                    Lista el contenido del archivo
  ecad extract <archivo.epdz> <carpeta>     Extrae el archivo en <carpeta>
  ecad inspect <archivo.epdz>               Resumen del proyecto
  ecad inspect <archivo.epdz> --json [out]  Proyecto completo en JSON
                                            (a la salida estándar o a <out>)
  ecad e3d2stl <entrada.e3d|.epdz> [--pieza <typeId/objectId>] [--todas] [-o <salida>]
                                            STL binario (mm, Z arriba). Con .epdz,
                                            un STL por espacio en la carpeta <salida>.
                                            Sin --todas, solo piezas físicas.
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match args.as_slice() {
        ["ls", path] => ls(path),
        ["extract", path, dir] => extract(path, dir),
        ["inspect", path] => open(path).map(|p| summary(&p)),
        ["inspect", path, "--json"] => open(path).and_then(|p| json(&p, None)),
        ["inspect", path, "--json", out] => open(path).and_then(|p| json(&p, Some(out))),
        ["e3d2stl", rest @ ..] => e3d2stl(rest),
        _ => {
            eprint!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ecad: {e}");
            ExitCode::FAILURE
        }
    }
}

type Result<T = ()> = std::result::Result<T, String>;

fn ls(path: &str) -> Result {
    let archive = Archive::open(path).map_err(|e| format!("{path}: {e}"))?;
    for entry in archive.entries() {
        println!("{:>10}  {}", entry.data.len(), entry.name);
    }
    Ok(())
}

fn extract(path: &str, dir: &str) -> Result {
    let archive = Archive::open(path).map_err(|e| format!("{path}: {e}"))?;
    let root = std::path::Path::new(dir);
    for entry in archive.entries() {
        // Solo rutas relativas sin `..`: un archivo no puede escribir fuera de <carpeta>.
        let rel = std::path::Path::new(&entry.name);
        if rel.is_absolute()
            || rel
                .components()
                .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err(format!("{}: ruta no permitida", entry.name));
        }
        let out = root.join(rel);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        std::fs::write(&out, &entry.data).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    println!("{} entradas extraídas en {dir}", archive.entries().len());
    Ok(())
}

fn e3d2stl(args: &[&str]) -> Result {
    use ecad_core::e3d::Scene;
    let (mut input, mut piece, mut all, mut out) = (None, None, false, None);
    let mut it = args.iter();
    while let Some(&a) = it.next() {
        match a {
            "--pieza" => piece = it.next().copied(),
            "--todas" => all = true,
            "-o" => out = it.next().copied(),
            _ if input.is_none() => input = Some(a),
            _ => {
                return Err(format!(
                    "argumento inesperado: {a}
{USAGE}"
                ));
            }
        }
    }
    let input = input.ok_or_else(|| USAGE.to_owned())?;
    let convert = |scene: &Scene, dest: &std::path::Path| -> Result<u32> {
        let parts = scene.parts.iter().filter(|p| {
            (all || p.kind().is_physical()) && piece.is_none_or(|k| p.key().as_deref() == Some(k))
        });
        let (stl, n) = ecad_core::stl::to_stl(scene, parts);
        if n > 0 {
            std::fs::write(dest, stl).map_err(|e| format!("{}: {e}", dest.display()))?;
            println!("{n:>8} triángulos  {}", dest.display());
        }
        Ok(n)
    };
    let path = std::path::Path::new(input);
    if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("e3d"))
    {
        let data = std::fs::read(path).map_err(|e| format!("{input}: {e}"))?;
        let scene = Scene::parse(&data).map_err(|e| format!("{input}: {e}"))?;
        let dest = out.map_or_else(|| path.with_extension("stl"), std::path::PathBuf::from);
        if convert(&scene, &dest)? == 0 {
            return Err("ninguna pieza coincide".into());
        }
        return Ok(());
    }
    let epdz = Epdz::open(path).map_err(|e| format!("{input}: {e}"))?;
    let dir = std::path::PathBuf::from(out.unwrap_or("."));
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut total = 0;
    for space in epdz.project().installation_spaces {
        let Some(file) = &space.file else { continue };
        let scene = epdz.scene(file).map_err(|e| format!("{file}: {e}"))?;
        let name = space.code.clone().unwrap_or(space.name.clone());
        let safe: String = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '.' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        total += convert(&scene, &dir.join(format!("{safe}.stl")))?;
    }
    if total == 0 {
        return Err("ninguna pieza coincide".into());
    }
    Ok(())
}

fn open(path: &str) -> Result<Project> {
    let epdz = Epdz::open(path).map_err(|e| format!("{path}: {e}"))?;
    Ok(epdz.project())
}

fn json(project: &Project, out: Option<&str>) -> Result {
    let text = serde_json::to_string_pretty(project).map_err(|e| e.to_string())?;
    match out {
        Some(out) => std::fs::write(out, text).map_err(|e| format!("{out}: {e}")),
        None => {
            let mut stdout = std::io::stdout().lock();
            writeln!(stdout, "{text}").map_err(|e| e.to_string())
        }
    }
}

fn summary(p: &Project) {
    let h = &p.header;
    let dash = |v: &Option<String>| v.clone().unwrap_or_else(|| "—".into());
    println!("Proyecto     {}", dash(&h.name));
    println!("Descripción  {}", dash(&h.description));
    println!("Número       {}", dash(&h.number));
    println!("Cliente      {}", dash(&h.customer.name).replace('\n', " "));
    println!("Empresa      {}", dash(&h.company));
    println!("Idioma       {}", dash(&h.language));
    println!("EPLAN        {}", dash(&h.eplan_version));
    println!();

    println!("{} páginas", p.pages.len());
    for page in &p.pages {
        println!(
            "  {:<20} {}",
            page.designation.full,
            page.description.as_deref().unwrap_or("")
        );
    }
    println!();

    println!("{} espacios de instalación", p.installation_spaces.len());
    for space in &p.installation_spaces {
        println!(
            "  {:<12} {:<10} {}",
            space.label.as_deref().unwrap_or(&space.name),
            space.description.as_deref().unwrap_or(""),
            space.file.as_deref().unwrap_or("(sin E3D)")
        );
    }
    println!();

    println!("{} conexiones", p.connections.len());
    let kinds = count(&p.connections, |c| c.kind);
    for (kind, n) in kinds {
        println!("  {n:>5}  {}", connection_kind(kind));
    }
    let routed = p.connections.iter().filter(|c| c.wire_3d.is_some()).count();
    let chained = p
        .connections
        .iter()
        .filter(|c| c.daisy_chain.is_some())
        .count();
    let ends = |f: fn(&Connection) -> bool| p.connections.iter().filter(|c| f(c)).count();
    println!("  {routed:>5}  enrutadas en 3D");
    println!("  {chained:>5}  en cadenas de puentes");
    println!(
        "  {:>5}  con tratamiento de extremo",
        ends(|c| c.source.treatment.is_some() || c.target.treatment.is_some())
    );
    println!();

    println!("{} funciones", p.functions.len());
    for (kind, n) in count(&p.functions, |f| f.kind) {
        println!("  {n:>5}  {}", function_kind(kind));
    }
    println!();

    println!("{} puntos de interrupción", p.interruption_points.len());
    println!("{} nodos de estructura", p.structure.len());
}

fn count<T, K: Ord>(items: &[T], key: impl Fn(&T) -> K) -> BTreeMap<K, usize> {
    let mut map = BTreeMap::new();
    for item in items {
        *map.entry(key(item)).or_insert(0) += 1;
    }
    map
}

fn connection_kind(kind: ConnectionKind) -> &'static str {
    match kind {
        ConnectionKind::Wire => "hilos",
        ConnectionKind::Bridge => "puentes",
        ConnectionKind::Tube => "tubos neumáticos",
        ConnectionKind::CableCore => "conductores de cable multipolar",
    }
}

fn function_kind(kind: FunctionKind) -> &'static str {
    match kind {
        FunctionKind::Device => "dispositivos",
        FunctionKind::Terminal => "bornes",
        FunctionKind::TerminalStrip => "regletas",
        FunctionKind::ConnectionPoint => "puntos de conexión",
        FunctionKind::Cable => "cables",
        FunctionKind::Placement => "colocaciones de artículo",
        FunctionKind::Mounting => "componentes de armario",
        FunctionKind::Topology => "rutas de topología",
    }
}
