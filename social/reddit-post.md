# Title

I built a free app to manage local domains with HTTPS - no more editing hosts files or self-signed cert warnings

# Body

I got tired of manually editing `/etc/hosts`, generating self-signed certificates, and dealing with browser warnings every time I needed a custom local domain for development. So I built **LocalDomain** - a native desktop app that handles all of it.

**What it does:**

- Create local domains like `myapp.local` or `api.myapp.local` that point to your dev server
- Automatic trusted HTTPS - no more `ERR_CERT_AUTHORITY_INVALID` warnings
- Built-in reverse proxy - map `myapp.local` to `localhost:3000`, `api.myapp.local` to `localhost:8080`, etc.
- Wildcard domain support for multi-tenant apps
- One-click setup wizard, no terminal commands needed

**How it works under the hood:**

- A lightweight background service manages your hosts file and runs a Caddy reverse proxy
- A local Certificate Authority is created and trusted by your OS, so browsers show a green lock
- The app talks to the service over IPC - you just click buttons

**Stack:** Tauri (React + Rust), Caddy, SQLite

Available for **macOS** and **Windows**. Completely free.

Download: https://getlocaldomain.com/

Would love to hear feedback or feature requests from the community.
