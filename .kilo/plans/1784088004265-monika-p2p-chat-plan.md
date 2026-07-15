# Monika — Decentralized P2P Team Chat

Private, serverless alternative to Slack/Discord. Fully decentralized (no central
servers), end-to-end encrypted, peers sync directly over the Tor network.

## Locked Decisions

| Area | Decision |
|------|----------|
| Delivery / offline | **P2P + DHT relay**: Tor-routed DHT provides peer discovery + encrypted store-and-forward. Messages held by nearby peers and delivered when recipients reconnect. |
| Tech stack | **Rust core + React**. Shared Rust core compiled to native libs via UniFFI. Desktop = Tauri + React/TS. Mobile = React Native reusing the same core + UI logic. |
| Group E2EE | **MLS (RFC 9420) via OpenMLS**. Group keying per channel; forward secrecy + post-compromise security. MLS "Delivery Service" = the DHT/P2P layer. |
| Identity | **Username + auto-generated "friend code"** (encoded public-key handle). Local password unlocks the device keystore. **No remote recovery** — forgot password = local reset (E2EE makes server recovery impossible). |
| Tor | **Embedded arti** in the Rust core; mobile falls back to Orbot/system Tor via SOCKS. |
| Phasing | **Desktop-first (Win/Mac/Linux via Tauri), then mobile (RN).** |

## Architecture

- **Rust core** (one crate, FFI via UniFFI):
  - `net`: Kademlia-style DHT adapted to run over Tor circuits (arti); peer discovery, store-and-forward of encrypted envelopes.
  - `crypto`: account keypair (ed25519/x25519) + per-device keypairs; local keystore encrypted by user password; BIP39 mnemonic backup of account key.
  - `mls`: OpenMLS wrappers for group (channel) create/join/remove, commit handling, message seal/open.
  - `sync`: fetch encrypted envelopes from DHT on connect, decrypt, persist, notify UI.
  - `invite`: encode/decode invite link/code.
- **Desktop (Phase 1):** Tauri shell + React/TS UI. Platform notifications via Tauri API.
- **Mobile (Phase 2):** React Native shell + embedded arti / Orbot fallback; background sync.

### Data / Invite model
- **Community** = set of MLS groups (one per channel) + membership roster.
- **Friend code** = encoded account public key (human-readable, e.g. base32 + checksum).
- **Invite code/link** = encoded blob: `{ community_id, bootstrap_peer_list (DHT rendezvous), MLS welcome/commit, join_token signed by an existing member }`.
- **Messages** = MLS-sealed envelopes stored encrypted in the DHT under content-addressed keys; only group members can decrypt.
- **Late joiners**: receive messages from join point onward by default (optionally a member re-shares history via an MLS external-join + ratchet export if desired — out of scope for MVP).

### Files / images
- Chunk files (>~1 MB), encrypt with a per-file symmetric key, store chunks as DHT envelopes, embed the key + manifest in an MLS-sealed message. Receiver reassembles + decrypts.

## Phase 1 — Desktop MVP (tasks)

1. **Scaffold**: Rust workspace + Tauri + React/TS app skeleton; CI builds for Win/Mac/Linux.
2. **Rust crypto**: account/device keys, password-protected keystore, BIP39 backup/restore of account key.
3. **Tor + DHT**: embed arti; implement Tor-routed Kademlia DHT with put/get of opaque encrypted envelopes; bootstrap via hardcoded + invite-provided peers.
4. **MLS**: OpenMLS integration — create channel group, add/remove member, seal/open messages, handle commits across the DHT.
5. **Invite flow**: generate + scan/enter invite code; new device discovers DHT, presents join token, receives MLS welcome, joins community/channels.
6. **UI**: communities list, channels, message thread, composer, image/file send + receive, desktop notifications on new messages.
7. **Sync engine**: on connect, pull pending envelopes, decrypt, persist locally (SQLite/IndexedDB-style store), update UI.
8. **Offline→online test**: verify messages sent while a peer is offline are delivered on reconnect.

## Phase 2 — Mobile
- React Native shell reusing Rust core via UniFFI; embedded arti / Orbot fallback.
- Background sync constraints, push-less local notifications, chunked file transfer over mobile Tor.

## Risks / Open Questions
- **DHT-over-Tor performance & relay reliability**: store-and-forward depends on enough online peers; needs redundancy/replication factor tuning.
- **MLS Delivery Service mapping**: commits must be reliably ordered/delivered — define envelope format + ordering.
- **Mobile Tor**: binary size, battery, background-network limits; Orbot fallback UX.
- **Sybil / spam / abuse**: open P2P networks are vulnerable; mitigate via invite-only communities (default) — full abuse tooling out of scope for MVP.
- **Metadata leakage**: Tor hides IP; DHT traffic patterns may leak metadata — accept as known limitation, revisit.
- **Repo name**: currently placeholder "Monika" — confirm product name.

## Validation
- Multi-node local simulation (≥3 desktop instances) exchanging messages over Tor.
- Offline-then-online delivery test (sender online, receiver offline, then reconnect).
- MLS member add/remove test (forward secrecy holds; removed member cannot read new messages).
- File/image round-trip (send → receive → decrypt → open).
- Tor-only connectivity test (no clearnet path used).
- BIP39 account-key restore test (wipe device, restore from seed, rejoin communities).
