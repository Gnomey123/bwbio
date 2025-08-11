# bwbio

A lightweight, native handler for the Bitwarden browser extension's biometric authentication.

Status: WIP/experimental. It works for me on Windows, but initial setup is still manual and there is no independent security audit. Use at your own risk.

## Why

The official Bitwarden browser integration relies on the Electron desktop app. On Windows this often leads to Windows Hello prompts appearing behind other windows, and it requires keeping another Electron app running in the background. bwbio is a tiny native executable written in Rust that:

- Focuses the Windows Hello dialog reliably to the foreground in my tests.
- Uses < 1 MB RAM while idle.
- Avoids Electron entirely.

## How it works

bwbio implements a Native Messaging host that speaks to the Bitwarden browser extension. It performs biometric-gated key release backed by Windows CNG + TPM:

- Keys are encrypted with an RSA-2048 key stored in the Platform Crypto Provider (TPM) via CNG.
- Windows Hello is used only for user presence verification (authentication), not for encryption/decryption. Once a process can access the TPM-resident key, it can decrypt the stored user key after a successful Windows Hello prompt.
- The host name is `com.8bit.bitwarden` and messages are exchanged over stdio per the Native Messaging protocol.

Security note: I am not a security professional. There has been no formal audit. All cryptography and key handling are best-effort and may contain mistakes. Please review before trusting with sensitive data.

## Features

- Native Messaging host for Bitwarden (Chrome/Edge/Brave/Arc variants supported via allowed origins).
- Biometric authentication via Windows Hello, auto-bringing the prompt to the foreground.
- Key management CLI to import/export/delete per-user Bitwarden keys, backed by CNG/TPM.

## Supported platform

- Windows 10/11. Linux/macOS are not supported.

## Build

Prerequisites:
- Rust toolchain (latest stable)

Build release binary:

```cmd
cargo build --release
```

The executable will be at `target\release\bwbio.exe`.

## Install and integrate with the browser extension (manual, WIP)

The following is a manual setup that works today. Automation is intentionally deferred while the project is WIP.

1) Create or ensure the TPM key exists (optional)
- By default, bwbio uses a CNG key named `bw-bio` in the Platform Crypto Provider.
- You can pre-create it using the CLI:

```cmd
# List existing CNG keys
bwbio.exe cng list

# Create the default key
bwbio.exe cng create bw-bio
```

2) Prepare the key storage directory
- By default, bwbio stores per-user encrypted keys next to the executable in a `keys` folder. You can override via env var `BW_KEY_DIR`.

3) Import your Bitwarden user key
- From the web vault or extension console, obtain your `userKeyB64` (base64). See `getkey.js` in this repo for hints on how to locate it programmatically; do this at your own risk.
- Import it for your Bitwarden userId:

```cmd
bwbio.exe import <userId> <userKeyB64>
```

- You can verify and test export (will trigger Windows Hello):

```cmd
bwbio.exe list
bwbio.exe check <userId>
bwbio.exe export <userId>
```

4) Register the Native Messaging host
- Copy `chrome.json` from the repo and update the `path` to the absolute path of your `bwbio.exe`:

```json
{
  "name": "com.8bit.bitwarden",
  "description": "Bitwarden desktop <-> browser bridge",
  "path": "C:\\full\\path\\to\\bwbio.exe",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://nngceckbapebfimnlniiiahkandclblb/",
    "chrome-extension://hccnnhgbibccigepcmlgppchkpfdophk/",
    "chrome-extension://jbkfoedolllekgbhcbcoahefnbanhhlh/",
    "chrome-extension://ccnckbpmaceehanjmeomladnmlffdjgn/"
  ]
}
```

- Place this manifest file in the location expected by your Chromium-based browser. For Chrome on Windows:
  - HKCU\Software\Google\Chrome\NativeMessagingHosts\com.8bit.bitwarden (Default) = REG_SZ with the full path to the JSON file.
  - Similarly for Edge: HKCU\Software\Microsoft\Edge\NativeMessagingHosts\com.8bit.bitwarden.

5) Launch path
- The program detects Native Messaging when argv[1] starts with `chrome-extension://` and switches to host mode automatically. Otherwise it runs the key manager CLI.

## Usage (CLI)

```text
bwbio.exe list                    # list stored Bitwarden user keys
bwbio.exe import <userId> <key>   # import a base64 user key for a user
bwbio.exe export <userId>         # export (biometric required)
bwbio.exe delete <userId>         # delete a stored key

bwbio.exe cng list                # list CNG keys in the Platform provider
bwbio.exe cng create <name>       # create an RSA-2048 key
bwbio.exe cng delete <name>       # delete a CNG key
```

Environment variables:
- CNG_KEY_NAME: override the CNG key name (default: bw-bio)
- BW_KEY_DIR: override where encrypted user keys are stored

## Caveats and security notes

- Windows Hello provides authentication only, not encryption; any local process with access to the TPM key can attempt decryption after user presence.
- No code audit; cryptography may be flawed. Treat as experimental.
- Native Messaging manifest and registry registration are manual at this stage.

## Credits

- https://github.com/quexten/bw-bio-handler
- https://github.com/quexten/goldwarden
- https://github.com/bitwarden/clients

## License

GPL-3.0-or-later. See LICENSE.
