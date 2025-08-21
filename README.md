# bwbio

A lightweight, native handler for the Bitwarden browser extension's biometric authentication.

TL;DR
bwbio is a tiny native helper that lets the Bitwarden browser extension trigger biometric unlocks via Windows Hello without requiring the Electron desktop app. Use on Windows 10/11; verify you understand the security notes before importing secrets.

Status: Ready for use. It works on Windows 10/11; there are still a few small rough edges and manual steps. No independent security audit has been performed — use at your own risk.

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

## IMPORTANT: permissions

Do NOT run bwbio as Administrator. Run bwbio as the same user that runs your browser. The program writes registry entries under HKCU (Current User) and must be run without elevated privileges; running under an Administrator account or elevated context can prevent correct CNG/TPM unlocking and will not register the host for other users.

## Quickstart — Interactive setup (recommended)

Download the latest release, double-click the included `bwbio.exe`, and choose Install. Then it will run the installed `bwbio.exe` and prompt you to import the key.

What the installer does: copies the exe to `%LOCALAPPDATA%\\bwbio`, writes `chrome.json`, and registers HKCU native messaging hosts.

## Importing keys

After installing the host, obtain two values from a logged-in Bitwarden web vault: the `userId` and the `userKey` (base64).

Open the Web Vault, open Developer Tools → Console (F12), paste the snippet below and run it; the console will print two lines: first `userId`, then `userKey` (base64). Copy them separately and paste into the interactive installer's Import prompts (first -> User ID, second -> User Key). Do NOT paste both values together.

```javascript
let userId = await this.bitwardenContainerService.keyService.stateService.getActiveUserIdFromStorage();
let masterKey = await new Promise(async r => (await this.bitwardenContainerService.keyService.masterPasswordService.masterKey$(userId)).subscribe(v => r(v)));
let userKey = await this.bitwardenContainerService.keyService.masterPasswordService.decryptUserKeyWithMasterKey(masterKey, userId);
console.log(userId);
console.log(userKey.keyB64);
```

### Clipboard & security

These values are sensitive secrets. Only run the console snippet on a trusted machine and browser. After copying, clear or manage your clipboard (Win+V) and avoid leaving these values in shared logs or screenshots.

## Manual install & registry

If you prefer a manual installation or need to place the manifest yourself, follow these fallback steps.

1) Copy the executable to your chosen install directory, for example:

```cmd
mkdir "%LOCALAPPDATA%\\bwbio"
copy bwbio.exe "%LOCALAPPDATA%\\bwbio\\bwbio.exe"
```

2) Create a `chrome.json` manifest in the install directory and update the `path` to the absolute path of your `bwbio.exe`:

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

3) Register the manifest by creating the following registry value(s) under HKCU for each browser you want to support:

- `HKCU\\Software\\Google\\Chrome\\NativeMessagingHosts\\com.8bit.bitwarden = <full path to chrome.json>`
- `HKCU\\Software\\Microsoft\\Edge\\NativeMessagingHosts\\com.8bit.bitwarden = <full path to chrome.json>`

Note: registry writes should be under HKCU (Current User). Do NOT attempt to write under HKLM or run registry tools elevated for the purpose of installing this host.

## Build

Prerequisites:
- Rust toolchain (latest stable)

Build release binary:

```cmd
cargo build --release
```

The executable will be at `target\\release\\bwbio.exe`.

## Uninstall

Recommended: run the interactive setup wizard and choose Uninstall. The wizard will attempt to:

- Remove the HKCU registry entries it created.
- Remove the `keys` directory under the install location.
- Remove the manifest file from the install directory.
- Attempt to delete the CNG key used by bwbio.

Manual uninstall (fallback):

- Delete `%LOCALAPPDATA%\\bwbio` (or your chosen install dir).
- Remove the manifest file and delete the registry values under HKCU described in Manual install.
- Optionally delete the CNG key via the CLI if needed.

## Troubleshooting

- If the console snippet fails or `this.bitwardenContainerService` is undefined, ensure you are on the Web Vault page. The snippet is a best-effort hint and may vary between versions.
- If import fails during export/check operations, verify that you ran bwbio without elevation and that the CNG key exists and is accessible under your user.

## Caveats and security notes

- Windows Hello provides authentication only, not encryption; TPM/CNG keys may be accessed without an additional per-operation confirmation, so once a process can access the TPM-resident key it can attempt decryption after user presence.
- No code audit; cryptography may be flawed. Treat as experimental.
- Native Messaging manifest and registry registration are per-user (HKCU). This program cannot register the host for all users.

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

## Credits

- https://github.com/quexten/bw-bio-handler
- https://github.com/quexten/goldwarden
- https://github.com/bitwarden/clients

## License

GPL-3.0-or-later. See LICENSE.
