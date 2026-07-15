# Monika

A decentralized, **serverless**, end-to-end encrypted team chat — a private
alternative to Slack/Discord. There are no central servers: peers discover and
sync with each other directly over the **Tor** network, and every message and
file is end-to-end encrypted.

> Status: **foundation implemented.** The security/identity core is built and
> tested. Networking (Tor + DHT), MLS group messaging, and the full UI are
> scaffolded but not yet implemented — see [Roadmap](#roadmap).

## Design (decided)

| Concern | Decision |
|---------|----------|
| Delivery / offline | P2P + Tor-routed DHT with encrypted store-and-forward |
| Tech stack | Shared Rust core, exposed via UniFFI; Tauri + React desktop, React Native mobile |
| Group E2EE | MLS (RFC 9420) via OpenMLS |
| Identity | Username + auto-generated "friend code"; local password; no remote recovery |
| Tor | Embedded `arti` (desktop), Orbot/system-Tor fallback (mobile) |
| Phasing | Desktop-first, then mobile |

See `.kilo/plans/` for the full plan.

## What is implemented

`crates/core` — the parts that need no live network, fully unit-tested:

- **Account / device keys** (`crypto/keys.rs`): Ed25519 account identity + X25519 device key.
- **Friend code** (`crypto/friend_code.rs`): checksummed, human-readable encoding of a public key (`MONIKA-XXXX-...`).
- **Mnemonic backup** (`crypto/keys.rs`): 24-word BIP39 recovery phrase deterministically derives the account key.
- **Encrypted keystore** (`crypto/keystore.rs`): account + device keys encrypted at rest with Argon2id + ChaCha20-Poly1305. Wrong password ⇒ auth failure.
- **Invite codes** (`crypto/invite.rs`): shareable community invite = encoded (community id, bootstrap peers, MLS welcome placeholder, inviter-signed join token) with encode/decode + signature verification.

`src-tauri` + `frontend` — Tauri v2 + React scaffold that wires the core into
the desktop shell (friend-code generation, invite create/verify, notifications).

## Building & running

### Core (works in this environment)

```sh
cargo test -p monika-core
```

### Desktop app (requires a normal dev machine)

Tauri needs platform system libraries (WebKitGTK on Linux, plus build tools).
Install them, then:

```sh
# one-time: generate app icons
cd src-tauri && npm run tauri icon <path-to-1024px-png>   # or add icons/ manually

# install JS deps and run
cd frontend && npm install
cd ../src-tauri && npm install
npm run tauri dev      # from src-tauri
```

Linux (Debian/Ubuntu) system deps:

```sh
sudo apt install libwebkit2gtk-4.1-dev build-essential \
  curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev \
  librsvg2-dev pkg-config
```

## Roadmap

1. **Tor + DHT** — embed `arti`; Kademlia-style DHT adapted for Tor circuits; encrypted store-and-forward envelopes.
2. **MLS** — OpenMLS integration for channel groups (create/join/remove, commits, seal/open) delivered over the DHT.
3. **Keystore persistence + sync engine** — load on unlock, fetch/decrypt envelopes on connect.
4. **Full UI** — communities, channels, messaging, image/file send, desktop notifications.
5. **Mobile** — React Native shell reusing the Rust core; embedded `arti` / Orbot.

## Security notes / known limitations

- Forgot password = **local reset**; E2EE makes server-side recovery impossible by design.
- DHT-over-Tor relay reliability, MLS delivery-service ordering, mobile Tor
  battery/background limits, and Sybil/spam mitigation are open problems tracked
  in the plan.
