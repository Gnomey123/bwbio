// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2025 Aalivexy

use bwbio::{browser::launch_native_messaging, cli::kmgr_cli, tui::tui_cli};
use std::{env::args, process::exit};

fn main() {
    if args()
        .collect::<Vec<_>>()
        .get(1)
        .is_some_and(|s| s.starts_with("chrome-extension://"))
    {
        launch_native_messaging().unwrap_or_else(|e| {
            eprintln!("Error launching native messaging: {e}");
            exit(1);
        });
        return;
    }

    if args().count() == 1 {
        tui_cli();
    } else {
        kmgr_cli();
    }
}
