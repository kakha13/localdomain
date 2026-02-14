# Title

Built a free app that gives you local domains + trusted HTTPS on any port

# Body

Every project I work on ends up needing a custom local domain at some point - testing OAuth callbacks, cookie sharing across subdomains, or just wanting `myapp.local` instead of `localhost:3000`. The setup was always the same tedious routine: edit hosts file, generate certs with mkcert, configure nginx or caddy, repeat.

So I built **LocalDomain** to automate all of that into a single app.

**You just:**

1. Open the app
2. Type a domain name and target port
3. Done - `https://myapp.local` works in your browser with a trusted certificate

**Features:**

- Custom local domains (`myapp.local`, `api.myapp.local`, etc.)
- Trusted HTTPS with auto-generated certificates - no browser warnings
- Reverse proxy built in - point any domain to any local port
- Wildcard support (`*.myapp.local`)
- Works on **macOS** and **Windows**

It runs a lightweight background service that manages your hosts file, a Caddy reverse proxy, and a local CA. Built with Tauri (React + Rust).

Completely free: https://getlocaldomain.com/

Happy to answer any questions or hear what features would be useful.
