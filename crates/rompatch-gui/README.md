# rompatch-gui

Native macOS GUI for rompatch. Thin Tauri 2 shell over `rompatch-core`,
with a React + Tailwind frontend.

## Layout

```
crates/rompatch-gui/
├── Cargo.toml            # Rust shell, pinned with `=X.Y.Z`
├── build.rs              # tauri_build::build()
├── tauri.conf.json       # window/bundle/security config
├── capabilities/         # Tauri 2 permission capabilities
├── icons/                # placeholder solid-color icons
├── src/                  # Rust IPC handlers
└── ui/                   # React + Vite frontend
    ├── package.json      # exact-pinned deps
    ├── .npmrc            # save-exact, save-prefix=""
    └── src/
```

## Dev

Requires `pnpm` 10.x and a stable Rust toolchain.

```bash
# Install frontend deps (lockfile is the source of truth).
cd crates/rompatch-gui/ui
pnpm install --frozen-lockfile

# From crates/rompatch-gui/:
cargo install tauri-cli --version 2.11.1 --locked  # one-time
cargo tauri dev
```

`cargo tauri dev` runs the Vite dev server on `localhost:1420` and
launches the windowed app with HMR.

## Build a `.dmg`

```bash
cd crates/rompatch-gui
cargo tauri build --target universal-apple-darwin
```

The unsigned `.dmg` lands in
`target/universal-apple-darwin/release/bundle/dmg/`. Recipients will
hit Gatekeeper on first launch ("can't be opened - Apple cannot check
it") and need to right-click - Open - Open. To produce a signed +
notarized `.dmg` that launches without that dance, cut a tag (see
[Releases](#releases)).

## Releases

Tag pushes (`v*`) trigger a universal `.dmg` build in CI, attach a
sigstore-backed build-provenance attestation, and publish a GitHub
Release with the `.dmg` attached.

Two flavours depending on whether Apple Developer secrets are present:

| State                        | Result                                                                                          |
|------------------------------|-------------------------------------------------------------------------------------------------|
| No Apple secrets (default)   | Unsigned + sigstore-attested `.dmg`. Recipients hit Gatekeeper on first launch (right-click - Open). |
| Apple secrets populated      | Codesigned, notarized, stapled, *and* sigstore-attested `.dmg`. No Gatekeeper prompt.            |

Sigstore attestation runs unconditionally. Apple steps activate the
moment the seven `APPLE_*` + `KEYCHAIN_PASSWORD` secrets land.

### Cutting a release

```bash
git tag v0.1.0
git -c core.sshCommand="ssh -i ~/.ssh/gwm-claude" push origin v0.1.0
```

CI takes ~10 minutes. On success, the release lands at
<https://github.com/GregTheGreek/rompatch-rs/releases>.

### Verifying a downloaded `.dmg`

Sigstore attestation (works on every release):

```bash
gh release download v0.1.0 --pattern "*.dmg" -D .
gh attestation verify rompatch_0.1.0_universal.dmg --owner GregTheGreek
# Loaded digest sha256:...
# Verified attestation. Provenance verified.
```

Apple signature + notarization (only when Apple secrets were set):

```bash
xcrun stapler validate rompatch_0.1.0_universal.dmg
# The validate action worked!

spctl --assess --type open --context context:primary-signature rompatch_0.1.0_universal.dmg
# accepted
# source=Notarized Developer ID
```

If the Apple checks pass, double-clicking the `.dmg` and dragging to
`/Applications` produces an app that launches without any Gatekeeper
prompt on any macOS 10.15+ machine.

### Enabling Apple signing (one-time setup)

When you're ready to switch from "attested only" to "attested +
codesigned + notarized":

1. Enrol in the [Apple Developer Program](https://developer.apple.com/programs/enroll/)
   ($99/yr).
2. In Keychain Access - Certificate Assistant - "Request a Certificate
   from a Certificate Authority". Save the CSR locally.
3. At <https://developer.apple.com/account/resources/certificates/list>,
   create a **Developer ID Application** cert from the CSR. Download
   the `.cer` and double-click to install in Keychain.
4. In Keychain Access, find `Developer ID Application: <name> (<TEAM_ID>)`.
   Right-click - Export as `.p12`. Set a strong password.
5. `base64 -i developer-id.p12 -o developer-id.p12.b64`. The contents
   of the `.b64` file are the `APPLE_CERTIFICATE_B64` secret.
6. At <https://appleid.apple.com> - Sign-In and Security - App-Specific
   Passwords, create one labelled `rompatch-notarize`. Copy it.
7. Note your **Apple Team ID** (10 chars, visible at
   <https://developer.apple.com/account>).

Then add these repo secrets at
<https://github.com/GregTheGreek/rompatch-rs/settings/secrets/actions>:

| Secret                       | Value                                                           |
|------------------------------|-----------------------------------------------------------------|
| `APPLE_CERTIFICATE_B64`      | Contents of `developer-id.p12.b64`                              |
| `APPLE_CERTIFICATE_PASSWORD` | The `.p12` export password                                      |
| `APPLE_SIGNING_IDENTITY`     | `Developer ID Application: <Your Name> (<TEAM_ID>)`             |
| `APPLE_ID`                   | Your Apple ID email                                             |
| `APPLE_PASSWORD`             | The app-specific password from step 6                           |
| `APPLE_TEAM_ID`              | 10-char team ID                                                 |
| `KEYCHAIN_PASSWORD`          | Any random string - the ephemeral CI keychain password          |

The CI workflow detects `APPLE_CERTIFICATE_B64` being non-empty and
flips automatically. Re-tag (or push a new tag) after adding secrets to
verify.

## Pinning policy

All third-party dependencies are pinned to exact versions. Updates only
happen via a dedicated PR that names the CVE or breaking change being
addressed.

| Surface              | Mechanism                                              |
|----------------------|--------------------------------------------------------|
| Cargo deps           | `=X.Y.Z` in `Cargo.toml`. `Cargo.lock` committed.      |
| Frontend deps        | Exact versions in `package.json` (no `^`, no `~`).     |
|                      | `.npmrc` sets `save-exact=true` to keep them that way. |
|                      | `pnpm-lock.yaml` committed; CI uses `--frozen-lockfile`. |

Audit at any time:

```bash
# Cargo: every version in this crate's manifest must start with `=`.
grep -E '"[0-9]' crates/rompatch-gui/Cargo.toml | grep -v '"='
# (no output = clean)

# Frontend: no carets or tildes in package.json.
grep -E '[\^~]' crates/rompatch-gui/ui/package.json
# (no output = clean)
```

## Icons

The committed `icons/*.png` are solid-colour placeholders, generated by
the Python snippet in this repo's history. To swap in a real branded
icon, drop a 1024x1024 source PNG in `assets/icon.png` and run:

```bash
pnpm tauri icon assets/icon.png
```

Tauri's CLI fans out platform-specific sizes (and `.icns` / `.ico`) into
`crates/rompatch-gui/icons/`.

## IPC command surface

All commands defined in `src/commands.rs`. The frontend wraps them in
`ui/src/lib/tauri.ts` for type safety.

| Command                  | Args                                                       | Returns                          |
|--------------------------|------------------------------------------------------------|----------------------------------|
| `detect_patch_format`    | `patchPath: string`                                        | `FormatKind \| null`             |
| `describe_patch`         | `patchPath: string`                                        | `PatchInfo`                      |
| `detect_rom_header`      | `romPath: string`                                          | `HeaderKind \| null`             |
| `compute_hashes`         | `filePath: string`                                         | `HashReport`                     |
| `apply_patch`            | `romPath, patchPath, outPath, options: ApplyOptions`       | `ApplyReport`                    |
| `default_output_path`    | `romPath: string`                                          | `string`                         |
