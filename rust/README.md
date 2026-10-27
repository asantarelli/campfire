# SDCancionero (versión Rust, Windows de 32 bits)

La misma idea que la versión Kotlin de este repositorio, escrita en Rust con [egui](https://github.com/emilk/egui), para las
computadoras de Windows de 32 bits donde la otra no corre. Un solo programa, sin Java, que arranca en una notebook vieja.

Lee y escribe **la misma biblioteca** que la versión Kotlin (`songs/*.cho`, `setlists/*.setlist.json` y el zip que exporta),
así que se puede pasar de una a otra con un zip.

## Qué hace

- **Canciones:** lista con búsqueda y orden (por título, por artista o agrupada por artista), botones con iconos, visor con acordes sobre la letra, columnas, tamaño y tipografía a elección (también
  monoespaciadas), transposición, autoscroll y diagramas de acordes (al tocar un acorde, o fijos al costado).
- **Acordes sobre su palabra:** cada acorde queda exactamente sobre el texto que lo seguía en el archivo; si dos acordes chocan,
  se separa la letra, nunca el acorde.
- **Editor:** botones para insertar acordes, anotaciones, secciones, comentarios y datos de la canción, menús de símbolos y
  rasgueos, panel de referencia ChordPro, transposición del texto y resaltado de la sintaxis.
- **Pegar:** una hoja de acordes copiada de una página (acordes sobre la letra, con encabezados `Verso`, `Estribillo`…) se
  convierte a ChordPro, tanto al pegar en el editor como sobre la lista de canciones.
- **Repertorios:** listas ordenadas de canciones, cada una con su transposición.
- **Herramientas:** metrónomo (también junto a la canción), afinador por micrófono con tono de referencia, acordes y escalas
  con su diapasón y los acordes que apila cada una.
- **Escuchar:** busca la canción en YouTube, YouTube Music o Spotify, en el navegador.
- **Zip:** importar y exportar la biblioteca completa.

## Compilar

Hace falta [Rust](https://rustup.rs) con el destino de 32 bits y las herramientas de compilación de Visual Studio:

```
rustup target add i686-pc-windows-msvc
cd rust
cargo test --workspace
cargo build --release --target i686-pc-windows-msvc -p sdcancionero
```

El ejecutable queda en `target/i686-pc-windows-msvc/release/sdcancionero.exe`. Lleva el runtime de C incluido, así que no pide
instalar nada más.

### Instalador

Con [NSIS](https://nsis.sourceforge.io) instalado:

```
powershell -ExecutionPolicy Bypass -File installer/build.ps1
```

Escribe `dist/SDCancionero-Rust-Setup-<versión>.exe`. Instala por usuario (sin administrador) en su propia carpeta, junto a la
versión Kotlin y sin pisarla, y nunca toca la biblioteca.

## Dónde guarda las cosas

- Biblioteca: `%LOCALAPPDATA%\SDCancionero\library` (`songs/` y `setlists/`).
- Ajustes: `%LOCALAPPDATA%\SDCancionero\settings.json`.

## Cómo está hecho

| Carpeta | Qué es |
|---|---|
| `chordpro/` | El núcleo, sin interfaz ni archivos: parser ChordPro, acordes, notación (inglesa, alemana, latina), transposición, conversión de hojas de acordes, edición de texto, escalas y posiciones de acordes. |
| `library/` | La carpeta de canciones y repertorios, los nombres de archivo y el zip. |
| `instruments/` | El metrónomo y el afinador como simples muestras de audio, sin placa de sonido. |
| `sdcancionero/` | La ventana (egui): visor, editor, herramientas, repertorios y ajustes; el audio con `cpal`. |
| `installer/` | El instalador de NSIS. |

El parser, la notación y la transposición son un port de los de la versión Kotlin, y las tablas de sus tests (82 acordes con sus
notas, el barrido de nombres reconocidos, los nombres alemanes y latinos) están portadas a `chordpro/tests`. Los tests no
necesitan ventana ni sonido: el metrónomo se prueba sobre diez minutos de muestras y el afinador sobre ondas sintéticas.

## Lo que no hace (todavía)

- Numeraciones Nashville y romana, y la transposición de las tablaturas.
- Los diagramas de `{define}`: los dibuja por búsqueda a partir del nombre del acorde.
- Leer PDF o Word al importar, ni los formatos de otras aplicaciones.
- Sincronización con la nube.

## Licencias

MPL-2.0, como el resto del repositorio. Lleva las tipografías Inter y JetBrains Mono (OFL), cuyas licencias están en
`presentation/src/commonMain/composeResources/files/licenses`. Las dependencias de Rust van enlazadas dentro del ejecutable;
sus licencias están en cada crate.
