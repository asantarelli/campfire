// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Puts the icon in the executable, so that the file and the taskbar show it.

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/icon.ico");
        resource.set("ProductName", "SDCancionero");
        resource.set("FileDescription", "SDCancionero");
        // An icon is not worth a failed build: without the resource compiler the executable simply has none.
        if let Err(error) = resource.compile() {
            println!("cargo:warning=No se pudo incrustar el ícono: {error}");
        }
    }
}
