# Sprint 1: Lectura del proyecto

**Objetivo:** `ecad-core` lee un `.epdz` y construye el modelo de proyecto a
partir de `manifest.db`, con todo lo que la v1 ya leía y lo que se dejaba fuera.

## Entregables

- `ecad_core::Epdz::open(path)`: extrae en memoria o a una carpeta temporal y
  clasifica páginas, E3D, AML y base de datos.
- `ecad_core::Project` con: cabecera, estructura (`=` / `+` / `&`), páginas,
  funciones (dispositivos y bornes), conexiones, puntos de interrupción,
  espacios de instalación.
- Conexiones completas según [modelo-datos.md](../modelo-datos.md): tratamiento
  de cada extremo, pelado, dirección de salida, recorrido, cadena de puentes,
  potencial, tipo (cable, puente, tubo).
- `ecad-cli inspect <archivo.epdz>`: resumen del proyecto y exportación a JSON.

## Tareas

- [x] Portar la extracción de `packages/e3d-core/src/epdz.ts`.
- [x] Portar `manifest.ts` a consultas `rusqlite` tipadas.
- [x] Añadir las propiedades que la v1 ignora (31051/31052, 31055/31056,
      31095, 31144/31145, 31148–31153, `source.*` / `destination.*`).
- [x] Investigar las propiedades sin identificar (31006, 31011, 31058, 31092,
      31114, 31115, 31155): analizadas con los datos del ejemplo y
      documentadas en [modelo-datos.md](../modelo-datos.md). **Falta compararlas
      con EPLAN**; mientras, se guardan sin mostrarse.
- [x] Normalizar designaciones (`=VA2+A1-FC1:6/T3` → función, emplazamiento,
      dispositivo, punto de conexión).
- [x] Clasificar conexiones: cable, puente, tubo neumático, cable multipolar.
- [x] Tests con `samples/ejemplo.epdz`: número de objetos, conexiones con
      origen y destino, textos con acentos bien decodificados.

## Criterios de aceptación

- `ecad-cli inspect samples/ejemplo.epdz` muestra 847 conexiones, 10 páginas
  y 7 espacios de instalación, y los textos con tildes salen bien.
- El JSON de salida contiene todos los campos de conexión del modelo.
- Ninguna propiedad marcada "por confirmar" se presenta como dato seguro.

## Estado

Hecho. `ecad_core::Epdz::open` + `Epdz::project()` construyen el modelo
(`crates/ecad-core/src/project.rs`) leyendo `manifest.db` en memoria, sin
fichero temporal. `ecad inspect samples/ejemplo.epdz` da 847 conexiones,
10 páginas y 7 espacios; `--json` exporta el modelo completo (4 MB). Los
criterios están en `crates/ecad-core/tests/project.rs`. El servidor ya ordena
y titula las páginas según el manifest.

Pendiente fuera del sprint: validar en EPLAN las propiedades probables y por
confirmar (dirección de salida, par de apriete, pelado, 31006…31155).

## Dependencias

Sprint 0.
