Hey everyone! Maker here.

I built LocalDomain because I was tired of the repetitive ritual every developer knows: editing `/etc/hosts`, setting up reverse proxies, generating SSL certificates, and then doing it all over again for the next project.

Tools like Laravel Valet solve this nicely - but only if you're in the Laravel ecosystem. I wanted something **stack-agnostic** that works for any local dev server, whether it's React on port 3000, a Go API on 8080, or anything else.

**What LocalDomain does:**

- Map custom local domains to any port (e.g., `myapp.test` -> `localhost:3000`)
- One-click HTTPS with auto-generated certificates trusted by your browser - no more "Your connection is not private" warnings
- Wildcard domain support (`*.myapp.test`)
- Built-in request inspector so you can see HTTP traffic hitting your local domains in real time
- Audit log of every change made

**How it works under the hood:**

A lightweight background daemon (managed via the app) handles hosts file updates, reverse proxying through Caddy, and TLS certificate generation. The app communicates with it over a Unix socket. Everything stays local on your machine - nothing is sent anywhere.

Built with Tauri (Rust + React), so it's fast, native, and lightweight - the whole app is just a few MB.

Currently macOS only (Intel + Apple Silicon). Windows support is on the roadmap.

This is v0.1 - I'd love to hear your feedback, feature requests, or pain points with local dev domains. What would make this a must-have in your workflow?
