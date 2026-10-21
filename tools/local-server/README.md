<!--
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
-->
# Servidor local de SDCancionero

Sirve la versión web desde la propia computadora, sin conexión y sin instalar nada: usa el PowerShell que trae Windows 10 (de 32 o de 64
bits). Está pensado para equipos donde la versión de escritorio no corre, como uno de 32 bits.

## Instalación

1. Copia a la computadora la carpeta del paquete: `serve.ps1`, `instalar.ps1`, `instalar.bat` y una carpeta `app` con el contenido de
   `app/web/build/dist/wasmJs/productionExecutable` (lo genera `./gradlew :app:web:wasmJsBrowserDistribution`).
2. Ejecuta `instalar.bat`. Crea un acceso directo en la carpeta de Inicio, para que el servidor arranque oculto cada vez que se
   inicia sesión, y lo inicia en el momento.
3. Abre `http://localhost:8080/` en el navegador (Vivaldi, Chrome o Edge) y, del menú del navegador, elige *Instalar* la página como aplicación.

## Qué hay que saber

- **La dirección importa.** La biblioteca se guarda en el navegador, ligada a `http://localhost:8080`. Cambiar el puerto, o abrir la página
  por otra dirección, muestra una biblioteca vacía. Si hace falta otro puerto, se pasa a `instalar.ps1 -Port 9000`.
- **Las copias de seguridad son importantes.** Limpiar los datos del sitio borra la biblioteca. *Ajustes → Biblioteca → Exportar la
  biblioteca como zip* la guarda en un archivo, y el mismo zip sirve para llevarla a otro dispositivo.
- **Actualizar** es reemplazar el contenido de la carpeta `app` por el de una compilación nueva. La página lo nota al abrirse.
- **Desinstalar** es borrar `SDCancionero (servidor).lnk` de la carpeta de Inicio (`shell:startup`) y cerrar el proceso `powershell` del servidor.
- El navegador necesita ser de 2023 o posterior (Chrome o Edge 119, Firefox 120).
