# Title

Built a free app that gives you local domains + trusted HTTPS on any port

# Body

Made a tool to skip the whole hosts file + mkcert + nginx dance when you need a local domain.

**LocalDomain** lets you point something like `myapp.local` to `localhost:3000` with trusted HTTPS — from a GUI, no config files.

**What it does:**

- Maps custom local domains to any port
- Auto-generates trusted TLS certs (local CA, no browser warnings)
- Built-in Caddy reverse proxy
- Wildcard support (`*.myapp.local`)
- macOS + Windows

Under the hood it's a Tauri app (React + Rust) with a background service that manages the hosts file, certs, and proxy.

Free: https://getlocaldomain.com/

Feedback welcome — curious what else would be useful.
