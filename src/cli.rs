// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2025 Aalivexy

use crate::cng::CngProvider;
use crate::cng::default_key_name;
use crate::kmgr::KeyManager;
use argh::FromArgs;
use std::env;
use std::path::PathBuf;
use windows_strings::HSTRING;

#[derive(FromArgs, PartialEq, Debug)]
/// Key management command line tool
struct KmgrCmd {
    #[argh(subcommand)]
    cmd: Command,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Command {
    List(ListCmd),
    Import(ImportCmd),
    Export(ExportCmd),
    Delete(DeleteCmd),
    Check(CheckCmd),
    Cng(CngCmd),
}

#[derive(FromArgs, PartialEq, Debug)]
/// List all keys
#[argh(subcommand, name = "list")]
struct ListCmd {}

#[derive(FromArgs, PartialEq, Debug)]
/// Import key
#[argh(subcommand, name = "import")]
struct ImportCmd {
    /// user id
    #[argh(positional)]
    user_id: String,
    /// plaintext key
    #[argh(positional)]
    key: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Export key (Require biometrics)
#[argh(subcommand, name = "export")]
struct ExportCmd {
    /// user id
    #[argh(positional)]
    user_id: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Delete key
#[argh(subcommand, name = "delete")]
struct DeleteCmd {
    /// user id
    #[argh(positional)]
    user_id: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Check if key exists
#[argh(subcommand, name = "check")]
struct CheckCmd {
    /// user id
    #[argh(positional)]
    user_id: String,
}

/// CNG provider commands
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "cng")]
struct CngCmd {
    #[argh(subcommand)]
    cmd: CngSubCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum CngSubCommand {
    List(CngListCmd),
    Create(CngCreateCmd),
    Delete(CngDeleteCmd),
}

#[derive(FromArgs, PartialEq, Debug)]
/// List all CNG keys
#[argh(subcommand, name = "list")]
struct CngListCmd {}

#[derive(FromArgs, PartialEq, Debug)]
/// Create a CNG key
#[argh(subcommand, name = "create")]
struct CngCreateCmd {
    /// key name
    #[argh(positional)]
    key_name: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Delete a CNG key
#[argh(subcommand, name = "delete")]
struct CngDeleteCmd {
    /// key name
    #[argh(positional)]
    key_name: String,
}

pub fn kmgr_cli() {
    let cmd: KmgrCmd = argh::from_env();
    let key_name = match env::var("CNG_KEY_NAME") {
        Ok(s) => HSTRING::from(s),
        Err(_) => default_key_name(),
    };
    let key_dir = env::var("BW_KEY_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            env::current_exe()
                .expect("Failed to get current exe path")
                .parent()
                .expect("Failed to get parent dir")
                .to_path_buf()
                .join("keys")
        });
    let kmgr = KeyManager::new(key_name, key_dir);
    match cmd.cmd {
        Command::List(_) => match kmgr.list_keys() {
            Ok(keys) => {
                if keys.is_empty() {
                    println!("No keys found.");
                } else {
                    for k in keys {
                        println!("Key: {k}");
                    }
                }
            }
            Err(e) => eprintln!("Failed to list keys: {e}"),
        },
        Command::Import(ImportCmd { user_id, key }) => match kmgr.import_key(&user_id, &key) {
            Ok(_) => println!("Key imported successfully."),
            Err(e) => eprintln!("Failed to import key: {e}"),
        },
        Command::Export(ExportCmd { user_id }) => match kmgr.export_key(&user_id) {
            Ok(k) => println!("{k}"),
            Err(e) => eprintln!("Failed to export key: {e}"),
        },
        Command::Delete(DeleteCmd { user_id }) => match kmgr.delete_key(&user_id) {
            Ok(_) => println!("Key deleted successfully."),
            Err(e) => eprintln!("Failed to delete key: {e}"),
        },
        Command::Check(CheckCmd { user_id }) => match kmgr.check_key_exists(&user_id) {
            Ok(true) => println!("Key exists."),
            Ok(false) => println!("Key does not exist."),
            Err(e) => eprintln!("Failed to check key: {e}"),
        },
        Command::Cng(cng_cmd) => {
            let provider = match CngProvider::new() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to open CNG provider: {e}");
                    return;
                }
            };
            match cng_cmd.cmd {
                CngSubCommand::List(_) => match provider.enum_keys() {
                    Ok(keys) => {
                        if keys.is_empty() {
                            println!("No CNG keys found.");
                        } else {
                            for k in keys {
                                println!(
                                    "Key: {}, Algorithm: {}",
                                    unsafe { k.pszName.display() },
                                    unsafe { k.pszAlgid.display() }
                                );
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to list CNG keys: {e}"),
                },
                CngSubCommand::Create(CngCreateCmd { key_name }) => {
                    match provider.create_key(HSTRING::from(key_name.as_str())) {
                        Ok(_) => {
                            println!("CNG key '{key_name}' created successfully.")
                        }
                        Err(e) => {
                            eprintln!("Failed to create CNG key '{key_name}': {e}")
                        }
                    }
                }
                CngSubCommand::Delete(CngDeleteCmd { key_name }) => {
                    match provider.open_key(HSTRING::from(key_name.as_str())) {
                        Ok(key) => match key.delete() {
                            Ok(_) => {
                                println!("CNG key '{key_name}' deleted successfully.")
                            }
                            Err(e) => eprintln!("Failed to delete CNG key '{key_name}': {e}"),
                        },
                        Err(e) => {
                            eprintln!("Failed to open CNG key '{key_name}': {e}")
                        }
                    }
                }
            }
        }
    }
}
