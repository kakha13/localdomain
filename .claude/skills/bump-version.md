---
name: bump-version
description: Bump the app version across all project files (package.json, Cargo.toml, tauri.conf.json)
user_invocable: true
argument: "<new_version> (e.g. 0.2.0)"
---

# Bump Version

Update the app version in all project files. The user provides the new version as an argument (e.g. `/bump-version 0.2.0`).

## Steps

1. Read the current version from `package.json`
2. Update version in these files (replace old version with new):
   - `package.json` — top-level `"version"`
   - `src-tauri/tauri.conf.json` — `"version"`
   - `src-tauri/Cargo.toml` — `version =`
   - `shared/Cargo.toml` — `version =`
   - `daemon/Cargo.toml` — `version =`
3. Run `cargo check --workspace` to regenerate `Cargo.lock`
4. Report what was changed

## Important

- Only update the `version` field in each file, not dependency version references
- Do NOT update `website/`, `AUDIT-REPORT.md`, or `package-lock.json`
- If no argument is provided, ask the user what the new version should be
- Validate the version looks like semver (e.g. `0.1.3`, `1.0.0`)
