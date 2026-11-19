# Modelo de datos

Qué trae un `.epdz` y qué hace la v2 con ello. Inventario hecho sobre
`samples/ejemplo.epdz` (proyecto de ejemplo de EPLAN, 6,5 MB comprimido).

## Contenido del archivo

| Ruta | Qué es | Muestra |
| --- | --- | --- |
| `manifest.db` | SQLite con la estructura y propiedades del proyecto | 4 226 objetos, 114 754 propiedades |
| `packages/pages/items/pagesvg/*.svg` | Páginas de esquema | 10 páginas |
| `packages/installationspaces/items/installationspacee3d/*.E3d` | Espacios de instalación en 3D | 7 |
| `packages/automationML/*.aml` | AutomationML (CAEX/XML) con el proyecto completo | 31 MB |

## manifest.db

Tablas: `package` (objetos), `property` (propiedades por objeto, por
`propid` EPLAN o por `propname`), `item` (ficheros del archivo) y tablas por
tipo (`function_package`, `mergedconnection_package`, `location_package`,
`page_package`, `installationspace_package`, `interruptionpoint_package`,
`container_package`, `propertydefinition_package`) más las relaciones
`page_functions` y `page_interruptionpoints`.

| Tipo de objeto | Nº | Uso en v2 |
| --- | --- | --- |
| `mergedconnection` | 847 | **Lista de cableado** (cables, puentes y tubos neumáticos) |
| `function` | 3 186 | Dispositivos y bornes; búsqueda; enlace con esquema |
| `location` | 67 | Estructura (`=` función, `+` emplazamiento) |
| `interruptionpoint` | 66 | Saltos de potencial entre páginas |
| `page` | 10 | Esquemas |
| `installationspace` | 7 | Armarios / placas |
| `container` | 7 | Agrupación de elementos |
| `project` | 1 | Cabecera del proyecto |

### Conexiones (`mergedconnection`)

| Propiedad | Significado | Ejemplo | Estado |
| --- | --- | --- | --- |
| ep.31019 / ep.31020 | Origen / destino | `=VA2+A1-FC1:6/T3` | Seguro |
| ep.31144 / ep.31145 | Origen / destino con dirección de salida | `+A1-FC2:1 HR` | Sufijo por confirmar (HL, HR, TL, TR, RH, RT, R, LT, LH, `undefined`). **No siempre van en orden origen/destino** (28 de 545 cruzadas): se asignan por designación |
| ep.31004 | Color | `WH` | Seguro |
| ep.31007 · ep.31002 · ep.31060 | Sección con unidad · valor · unidad | `0,25 mm²` | Seguro |
| ep.31003 · ep.31000 | Longitud con unidad · en metros | `0,18 m` | Seguro |
| ep.31048 | Tipo de conductor / tubo | `PUN-H-12X2-BL` | Seguro |
| ep.31051 / ep.31052 | Tratamiento del extremo origen / destino | `End sleeve`, `Stripping` | Seguro. **La v1 no lo muestra** |
| ep.31055 / ep.31056 | Longitud de pelado origen / destino | `10 mm` | Probable. **No se muestra** |
| ep.33003 · ep.20045 | Potencial / señal | `PE`, `24 VDC PLC` | Seguro |
| ep.31095 | Canaleta o recorrido | `+B3.X1-U106` | Probable. **No se muestra** |
| ep.31148–31150 · ep.31151–31153 | x, y, z del origen · x, y, z del destino (mm) | `375.2` | Probable. Orden comprobado: el destino de un tramo coincide con el origen del siguiente en las cadenas de puentes |
| ep.31011 | Designación de la conexión (número de hilo / tubo) | `13-ø` | Probable. Solo en los 25 tubos del ejemplo |
| ep.20006 | Cable multipolar al que pertenece el conductor | `+A2-WD8` | Seguro. `+` si no va en cable |
| ep.20188 | Definición de función de la conexión | `4/1/4` | Probable: `4/1/4` hilo, `4/1/7` y `4/1/9` puentes, `4/1/15` tubo, `4/1/10` y `4/1/17` internas del dispositivo |
| ep.31058 | Siempre `1` en conductores de cable multipolar (209 = los de ep.20006) | `1` | Sin identificar |
| ep.31006 | Valores 2–7, en 207 conexiones de todo tipo | `3` | Sin identificar |
| ep.31092, 31114, 31115 | Enteros pequeños (1–34, 1–11, 1–12), solo en conexiones enrutadas; los tubos llevan 31092 = 34 | `29` | Sin identificar |
| ep.31155 | Entero único por conexión enrutada (544) | `525` | Sin identificar; ¿orden de enrutado? |
| `source.35575` / `destination.35575` | Par de apriete (?) | `2.5` | Por confirmar |
| `source.35576` / `destination.35576` | Tamaño de destornillador / boca (?) | `3` | Por confirmar |
| `daisychainid` · `daisyChainorderid` | Cadena de puentes y orden | `9` · `7` | Seguro |
| `connection3did` | Pieza 3D del cable enrutado | `166/60765` | Seguro |
| `connectionoid` | Conexiones EPLAN originales agrupadas | `253/233/61888/0` | Seguro |

Clasificación (`ecad_core::project::ConnectionKind`): **tubo** si ep.20188 es
`4/1/15`, el tipo empieza por `PUN` o la sección es un diámetro (`8mm`);
**conductor de cable** si ep.20006 nombra un cable; **puente** si origen y
destino son del mismo dispositivo; **hilo** el resto. En el ejemplo: 581 hilos,
32 puentes, 25 tubos y 209 conductores de cable.

En el modelo, lo probable y lo por confirmar va envuelto con su estado
(`{"value": …, "certainty": "probable" | "unconfirmed"}`) y lo sin identificar
se guarda en bruto en `unidentified`.

Las propiedades "por confirmar" se validan contra una exportación conocida en
EPLAN antes de mostrarlas; mientras tanto se guardan pero no se enseñan.

### Proyecto

ep.10000 nombre, ep.10011 descripción, ep.10013 número, ep.10015 cliente,
ep.10042 año, ep.10100 empresa, ep.3010/3011 equipo y usuario que exportó,
`language` idioma del proyecto.

## AutomationML

XML de 31 MB; se lee en streaming (`quick-xml`), nunca entero en memoria.

| Dato | Atributos AML | Uso |
| --- | --- | --- |
| Identificación | `Full item designation`, `Function designation…` | Enlazar con el manifest |
| Artículo | `Part number`, `Manufacturer name`, `product article number of manufacturer`, `GTIN`, `Manufacturer product description`, `Weight` | Lista de material, picking con lector de códigos |
| Ficha técnica | `Address of additional link` | Enlace a la ficha del fabricante |
| Posición | `Frame` (x, y, z, rx, ry, rz), `Position (in mm)` | Vista de placa, orden de montaje |
| Superficie de montaje | `Startpoint/Size of mounting surface`, direcciones normal y x | Qué placa, qué cara |
| Dimensiones | `width`, `height`, `depth`, `length`, `Diameter` | Listas de corte (carril, canaleta) |
| Taladros y fijación | `Hole, diameter`, `Fixing variant`, `Mounting point*n` | Plano de mecanizado |
| Conectores | `Connector*n`, `Type of connection point` | Bornes y puntos de conexión |
| Textos | `aml-lang=xx-XX` (19 idiomas) | Mostrar en el idioma del puesto |

## E3D

Formato binario propio de EPLAN (v1–v4), ya descifrado en la v1
(`packages/e3d-core/src/reader.ts`, `docs/formats.md`). En la v2 se usa para:

- **Vista frontal 2D**: proyección ortográfica de cada placa a SVG en el servidor.
- **Exportar a STL**: por pieza o por espacio de instalación.
- **Localizar**: cada pieza lleva `typeId/objectId`, que enlaza con el manifest.

## Datos propios de la v2 (servidor)

```
usuario(id, nombre, rol: oficina | taller | supervisor, activo)
proyecto(id, numero, nombre, cliente)
revision(id, proyecto_id, n, fichero_epdz, subida_por, subida_en,
         estado: borrador | publicada | obsoleta, notas)
orden(id, revision_id, espacio_instalacion, creada_en)     -- un armario a fabricar
estado_extremo(orden_id, conexion_clave, extremo: origen | destino,
               estado: pendiente | hecho | comprobado | desmontar | retirado,
               usuario_id, cuando)
estado_pieza(orden_id, pieza_clave, estado, usuario_id, cuando)
comentario(id, orden_id, objeto_clave, usuario_id, texto, cuando, resuelto)
evento(id, orden_id, usuario_id, tipo, datos_json, cuando)  -- registro inmutable
```

`conexion_clave` y `pieza_clave` son claves estables entre revisiones
(origen + destino normalizados, o el identificador EPLAN cuando es fiable), para
que el trabajo hecho sobreviva a una revisión nueva. Su definición se cierra en
el sprint 7.
