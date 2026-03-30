---
name: website-version-change
description: Use when updating website download links to a new release version for mac, windows, linux, or all platforms
user_invocable: true
argument: "<version> <platform> — version like 0.2.0, platform is mac|windows|linux|all (default: all)"
---

# Website Version Change

Update SourceForge download links on the website landing page and docs.

## Arguments

- First arg: new version (e.g. `0.2.0`). If omitted, ask the user.
- Second arg: platform — `mac`, `windows`, `linux`, or `all` (default: `all`)

Examples: `/website-version-change 0.2.0 mac`, `/website-version-change 0.2.0 all`, `/website-version-change 0.2.0`

## Files to Update

1. `website/pages/index.tsx` — landing page download buttons
2. `website/pages/docs/index.mdx` — docs download table

## URL Patterns per Platform

**mac:**
- URL: `https://sourceforge.net/projects/localdomain/files/v{VERSION}/LocalDomain_{VERSION}_aarch64.dmg`
- Display text (docs): `LocalDomain_{VERSION}_aarch64.dmg`

**windows:**
- URL: `https://sourceforge.net/projects/localdomain/files/v{VERSION}/LocalDomain_{VERSION}_x64_en-US.msi`
- Display text (docs): `LocalDomain_{VERSION}_x64_en-US.msi`

**linux:**
- URL: `https://sourceforge.net/projects/localdomain/files/v{VERSION}/LocalDomain_{VERSION}_amd64.deb`
- Display text (docs): `LocalDomain_{VERSION}_amd64.deb`

## Steps

1. Read current versions from `website/pages/index.tsx` by finding the SourceForge URLs
2. For each selected platform, use Edit to replace the old version in both files:
   - In `index.tsx`: update the `href` URL
   - In `docs/index.mdx`: update both the link text and the `href` URL
3. Report what was changed (old version -> new version, which platforms)

## Important

- Only update the platforms the user specified (or all if not specified)
- Each platform may be at a different version — detect the current version per-platform from the URLs
- Validate the version looks like semver (e.g. `0.1.3`, `1.0.0`)
- Do NOT modify any other files
