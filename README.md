<!--
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
-->
# SDCancionero
*Tu cancionero, en todas tus pantallas.*

SDCancionero es un visor y editor de hojas de acordes basado en el formato de texto plano [ChordPro](https://www.chordpro.org/chordpro/chordpro-introduction/).
Es un fork de [Campfire](https://github.com/pandulapeter/campfire), de Péter Pandula, adaptado al español y con su propia identidad.

<img src="documentation/screenshots/01.webp" width="32%" /> <img src="documentation/screenshots/02.webp" width="32%" /> <img src="documentation/screenshots/03.webp" width="32%" />

Permite organizar una biblioteca de canciones, armar repertorios y tocar en vivo. Las hojas de acordes se pueden pegar, compartir desde otras
aplicaciones o importar desde PDF, Word y copias de seguridad de otros cancioneros, y se convierten a ChordPro en el propio dispositivo.
Las canciones se adaptan a la pantalla, se transponen al vuelo, tienen una vista de solo letra y muestran diagramas de acordes para guitarra,
ukelele, mandolina, banjo y teclado. Incluye metrónomo, afinador y una biblioteca de escalas y modos, y los repertorios y canciones se
exportan a PDF para imprimir.

La aplicación funciona sin conexión, no necesita una cuenta ni depende de ningún servidor.

## Plataformas

| Plataforma | Cómo se obtiene |
|---|---|
| Android | `./gradlew :app:android:assembleRelease` (necesita el SDK de Android) |
| Windows (64 bits) | `./gradlew :app:desktop:packageMsi` |
| Web | `./gradlew :app:web:wasmJsBrowserDistribution`, y se sirve la carpeta `app/web/build/dist/wasmJs/productionExecutable` por `https` o `localhost` |

Compose Multiplatform no ofrece escritorio para Windows de 32 bits. En un equipo de 32 bits se usa la versión web desde un navegador
Chromium reciente (Chrome, Edge, Vivaldi): se puede instalar como aplicación y funciona sin conexión después de la primera visita.
Sin hosting: [`tools/local-server`](tools/local-server/README.md) sirve la versión web desde la propia computadora y arranca solo al iniciar sesión.

## Sincronizar entre dispositivos

*Ajustes → Biblioteca → Exportar la biblioteca como zip* genera un archivo con todas las canciones y repertorios. En el otro dispositivo,
*Importar canciones, documentos o archivos zip…* lo lee y permite elegir, para todos los conflictos a la vez, entre conservar ambas
versiones o reemplazar la que ya existe.

## Bajo el capó

Es un proyecto de [Kotlin Multiplatform](https://kotlinlang.org/docs/multiplatform.html) para Android, escritorio (JVM) y web (Kotlin/Wasm).
Toda la interfaz está escrita una sola vez con Compose Multiplatform y Material 3 Expressive. El código se divide en módulos pequeños con una
frontera estricta `api` / `implementation` en cada capa; la arquitectura y las convenciones están en [`CLAUDE.md`](CLAUDE.md) y en los
`CLAUDE.md` de cada módulo. Todo lo que guarda la aplicación es un archivo plano: las canciones son archivos `.cho` y los repertorios,
pequeños documentos JSON.

## Licencia

Copyright (c) Péter Pandula 2017-2026. Este software se distribuye bajo la
[Mozilla Public License 2.0](LICENSE).

La MPL es una licencia de copyleft a nivel de archivo: se puede reutilizar cualquier parte del proyecto en otro, de código abierto o no. Si se
modifica un archivo de este repositorio, ese archivo debe seguir bajo la MPL y su código fuente debe estar disponible; lo nuevo que se escriba
junto a él pertenece a quien lo escribe, con la licencia que elija.
