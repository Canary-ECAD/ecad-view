//! `ecad`: herramientas de línea de comandos sobre exportaciones de EPLAN.

use std::process::ExitCode;

const USAGE: &str = "\
Uso:
  ecad ls <archivo.epdz>    Lista el contenido del archivo
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["ls", path] => match ecad_core::Archive::open(path) {
            Ok(archive) => {
                for entry in archive.entries() {
                    println!("{:>10}  {}", entry.data.len(), entry.name);
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("ecad: {path}: {e}");
                ExitCode::FAILURE
            }
        },
        _ => {
            eprint!("{USAGE}");
            ExitCode::from(2)
        }
    }
}
