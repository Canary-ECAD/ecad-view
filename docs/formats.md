# File format notes

Community documentation of the EPLAN file formats supported by this project.
Everything here was derived from inspecting publicly distributed files; there
is no official specification. Corrections and additions are welcome.

## `.epdz` — project export archive

A `.epdz` file is a standard **7-zip archive** (magic `7z¼¯'`), *not* a ZIP.
Typical contents:

```
manifest.db                                   SQLite 3 project database
packages/
  pages/items/pagesvg/*.svg                   schematic pages (SVG 1.1)
  pages/items/pagesvg/SVGScripting.js         EPLAN's page interactivity script
  pages/items/pagesvg/EPLAN.png               images referenced by pages
  installationspaces/items/installationspacee3d/*.E3d   3D models
  automationML/*.aml                          project structure (AutomationML/XML)
```

### `manifest.db` (SQLite)

Schema version `1.0.0` (table `database`). Key tables:

| Table | Purpose |
| --- | --- |
| `package (id, parentid, type, name)` | The project tree. `type` ∈ `project`, `container`, `page`, `function`, `location`, `installationspace`, `propertydefinition`. |
| `page_package (packageid, name, ep1340, ep1440, ep1140, ep1240, ep1640, ep1540)` | Per-page structured identifier split into EPLAN properties: functional assignment / higher-level function / installation site / location (`==`/`++` segments), page counter (`#01`) and representation type (`&Multi_line`, `&EFS`, …). `name` matches the SVG file base name. |
| `item (packageid, type, locator, base, filesize, …)` | Maps packages to files inside the archive (`locator` is the file name, `type` e.g. `pagesvg`, `installationspacee3d`). |
| `property (packageid, propname, propid, value)` | Property bag. On the project package: `propid` 10011 = project description, 3011 = creator, 10016/10017 = company address, 2000 = EPLAN version, plus named entries like `language`, `pages.treeconf`. |
| `installationspace_package (packageid, name)` | 3D installation spaces; their `item` rows point to the `.E3d` files. |
| `function_package`, `location_package`, `container_package`, `project_package` | Names per package type. |
| `page_functions`, `page_interruptionpoints` | Page ↔ function / interruption-point relations (cross-references). |

### Schematic SVG pages

- SVG 1.1 with a `viewBox` in **millimetres** of the drawing frame (e.g.
  `0 0 420 297` for A3 landscape) and a global `transform` flipping the Y
  axis (`matrix(1,0,0,-1,0,H)`).
- The page title is in `<title>` in EPLAN structured syntax:
  `==EES==Page_macros++Infrastructure++Power_supply++AC400V&EFS#01/1`.
- Styling via CSS classes in an embedded `<style>` block (`.P0`, `.F3`, …).
- Interactivity via `<script xlink:href="SVGScripting.js">` plus
  `javascript:jumpToFunction(...)` anchor links for cross-references, and
  URL parameters (`search`, `highlightId`, `jumpurl`) parsed by the script.
  A standalone viewer must sanitize or reimplement these.
- Images (company logo) are referenced relatively (`xlink:href="EPLAN.png"`)
  and shipped inside the archive.

## `.e3d` — binary 3D scene

Little-endian binary. Producers: EPLAN Data Portal macros
(`…_3D.ema` → E3D download endpoint) and `.epdz` installation spaces.
Format versions 1–4 are covered here (a version byte > 4 exists in newer
EPLAN releases and is not yet documented).

### Primitive types

| Type | Encoding |
| --- | --- |
| `byte` | `uint8` |
| `short` | `int16` LE |
| `long` | `int32` LE |
| `float` | `float32` LE |
| `color` | 4 bytes RGBA where **A is transparency** (opacity = `1 − a/255`), each channel `/255` |
| `vector` | 3 × `float` |
| `transform` | 4 × `vector` = column-major 3x4 (rotation basis + translation), expands to a 4x4 with `[0,0,0,1]` |
| `buffer` | `long` byte-length prefix + raw bytes (`float[]`, `uint16[]`, `uint8[]`) |
| `string` | `long` byte-length prefix + UTF-16LE code units, NULs skipped |
| `array<T>` | `long` element-count prefix + `count` × `T` |

### File layout

```
byte      formatVersion                (1..4)
transform viewTransform                initial camera, see "Observed semantics"
vector    viewParams                   meaning unknown, see "Observed semantics"
color     backgroundTop
color     backgroundBottom
array<Light>    lights
array<Texture>  textures
array<Mesh>     meshes
array<Part>     parts
EOF (parsers should verify full consumption)
```

**Light** — `byte type` (0 ambient, 1 directional, 2 point, 3 spot),
`color`; position for types 2/3; direction for types 1/3; `float angle`,
`float exponent` for type 3.

**Texture** — `byte pixelFormat`, `short width`, `short height`, then
`width × height × 4` bytes of **BGRA** (swap R/B for RGB use).

**Mesh**

```
byte  vertexFlags        bit0 positions, bit1 normals, bit2 texcoords
floatbuffer vertexArray  interleaved [pos.xyz][normal.xyz][uv] per flags
array<FaceGroup>  faces
array<EdgeGroup>  edges
```

- `FaceGroup` = `Material` + `Elements`
- `EdgeGroup` = `EdgeStyle` + `Elements`
- `Material` = `byte colorValid` (0 → inherit the part color), `color`,
  `short textureId` (−1 = none)
- `EdgeStyle` = `color`, `float lineWidth`, `byte lineType`
- `Elements` =
  - v≥2 only: `vector center` (name kept for compatibility; it is not the
    centroid of the group, see "Observed semantics")
  - `byte mode` — OpenGL-style: 0 points, 1 lineStrip, 2 lineLoop, 3 lines,
    4 triangleStrip, 5 triangleFan, 6 triangles
  - `long indexCount`
  - `byte indexType` (0 = `uint8`, 1 = `uint16`) + index buffer

**Part** (an instance of a mesh)

```
short     meshId
transform placement
color     partColor            fallback for materials with colorValid=0
v3+: short typeId, long objectId    (ids linking into the project database)
v4+: array<TextLine> textLines
```

**TextLine** — `transform`, `float height`, `short justification` code
(row: 1–3 bottom / 4–6 middle / 7+ top; column within the row: left, center,
right), `string text`. Rendered height ≈ `0.75 × height`.

### Observed semantics

Findings from the seven installation-space files of `samples/ejemplo.epdz`
(all format version 4), cross-checked against the archive's `manifest.db` and
the geometry itself. Each item says how solid it is.

**Structure (confirmed).** The layout above consumes every byte of all seven
files. No index is out of range, no triangle is degenerate and every stored
normal is unit length. All face groups use `triangles` and all edge groups use
`lines`, always with `uint16` indices.

**`viewTransform` is the initial camera, not a model transform (strong).** In
every file the rotation part is the same isometric view (45° about Z, about 30°
tilt) and its third basis vector equals the direction of the directional light,
`(0.61, −0.61, 0.5)`: a headlight looking along the view axis. Only the
translation changes per file. Do not apply it to the geometry; part transforms
already place each part in world coordinates.

**`viewParams` (unknown).** Values such as `(−568, 5571, 394)` do not match the
scene's bounding box; files with the same content share the same values. They
look like camera parameters (clipping planes or distance), unproven.

**`Elements.center` (unknown).** It is neither the mean nor the bounding-box
centre of the group's vertices (deviations up to ~380 mm). Keep it raw and do
not rely on it.

**Lights (confirmed for these files).** Always two: one directional (the
headlight above) and one ambient.

**Part colour alpha.** Only three values occur: 0 (opaque), 128 (50 %
transparent) and 204 (80 % transparent). They follow the part's role (below),
so they look like a display convention rather than per-part data.

**Normals.** About half of the meshes (617 of 1 214) carry positions only
(`vertexFlags = 1`). Consumers that need normals (STL, shading) must compute
them from the faces.

**`typeId` identifies the kind of object (strong for most rows).** Parts link
to the project through `typeId/objectId`; the manifest stores the same pair
as `…/<typeId>/<objectId>/…` in property `2000`, and routed wires as
`connection3did`.

| typeId | Parts | Role | Evidence |
| --- | --- | --- | --- |
| 166 | 580 | Routed wire | Matches the 580 `mergedconnection.connection3did` values one to one |
| 180 | 1 151 | Device | Manifest `function`; text lines such as `-FC2:1`; 202 of them at 50 % alpha |
| 173 | 33 | Cable duct | Manifest `function`, tags `-U4`…; e.g. 1 470 × 60 × 80 mm; 50 % alpha |
| 178 | 21 | Mounting rail (DIN) | Manifest `function`; 35 × 15 mm cross-section |
| 174 | 24 | Busbar | Manifest `function`, tags `-WE1`…; 15 × 5 mm cross-section |
| 175 | 1 237 | Mounting surface (probable) | Flat (≤ 2 mm), 80 % alpha, not in manifest; the AML holds the same 1 237 `…/175/…` ids |
| 176 | 7 | Unknown | Plates and 11 × 11 × 10 mm pieces; 307 `…/176/…` ids in the AML |
| 185 | 77 | Unknown | Planar line-like shapes (one dimension 0) |
| 187, 238 | 1 each | Unknown | |

**Text lines.** Only devices, ducts, rails and busbars carry them (device tags
and terminal designations). Justification codes seen map to left/centre/right
on the middle row, plus a few bottom/top ones. The `+0.1` z offset in the
viewer is a rendering choice, not file data.

**Open.** Format versions above 4 (newer EPLAN releases) have not been seen
yet; a sample is needed before documenting them.

### Coordinate system

E3D uses **Z-up**; rendering in three.js (Y-up) requires rotating the root
by −90° around X (equivalently the matrix
`[1,0,0,0, 0,0,-1,0, 0,1,0,0, 0,0,0,1]` column-major).
