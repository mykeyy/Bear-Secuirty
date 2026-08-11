# Changelog

## 1.0.0 - 2026-08-11

Final pre-archive release.

- Rewrote Bear Security from Python to Rust.
- Replaced `discord.py` with Twilight 0.17.
- Added Discord Components V2 for the security panel and bot notices.
- Added the legacy `b!` prefix for `b!ping`, `b!about`, and `b!security`.
- Added configurable anti spam and anti scam message cleanup.
- Added an optional honeypot channel with moderator and bot exemptions.
- Added welcome and leave messages.
- Added optional autorole support.
- Added runtime storage selection for in-memory, Turso/libSQL, and PostgreSQL backends.
- Added a security policy, Rust toolchain pin, Nix development shell, and CI checks.
- Removed the old Python implementation and Python dependency files.

## 0.1.0 - 2025-05

Original Python prototype.

- Added `/ping` and `/bear` slash commands.
- Added uptime and latency reporting.
- Added a Nix development environment.
- Added project screenshots and the first README.
