<div align="center">
  <img src="./Image/bear.png" alt="Bear Security logo" width="160" />

  <h1>Bear Security</h1>

  <p>A small Discord moderation and security bot rewritten in Rust before being archived.</p>

  <p>
    <img alt="Rust 1.89" src="https://img.shields.io/badge/Rust-1.89-000000?logo=rust&logoColor=white" />
    <img alt="Twilight 0.17.1" src="https://img.shields.io/badge/Twilight-0.17.1-5865F2" />
    <img alt="Discord Components V2" src="https://img.shields.io/badge/Discord-Components%20V2-5865F2?logo=discord&logoColor=white" />
    <img alt="Turso" src="https://img.shields.io/badge/Storage-Turso%20%7C%20PostgreSQL-4FF8D2" />
    <img alt="MIT License" src="https://img.shields.io/badge/License-MIT-green" />
    <img alt="Final release" src="https://img.shields.io/badge/Release-1.0.0-lightgrey" />
  </p>
</div>

> [!IMPORTANT]
> Bear Security is a legacy project. Version `1.0.0` is the final pre-archive release. It is kept public as a reference, not as an actively maintained moderation product.

## A small note

I had not used this project in quite a while. The original version was a tiny Python Discord bot with `/ping` and `/bear`, and that was basically it.

Before archiving the repository, I did not want to leave something called "Bear Security" that did not really have any security features. I gave it one final rewrite in Rust and added the small moderation features I originally would have expected from the name.

It is still intentionally small. There is no giant moderation framework here. If you find this repository years from now and want to use it, treat it as a starting point. Discord's API, gateway rules, database clients, and library versions can change. Some parts may need updates before they work again.

## What it does

| Feature | Behavior |
| --- | --- |
| Anti spam | Removes fast message bursts and repeated copy-paste spam. |
| Anti scam | Removes suspicious giveaway and phishing style messages when scam wording is combined with a link. |
| Honeypot | Creates a clearly labelled bait channel. A non-moderator human who posts there can be kicked automatically. |
| Welcome messages | Sends a Components V2 welcome message when a member joins. |
| Leave messages | Sends a Components V2 leave message when a member leaves. |
| Autorole | Optionally assigns one configured role to new members. |
| Security panel | Uses Discord Components V2 with secondary gray buttons for moderation toggles. |
| Persistent settings | Supports Turso/libSQL or PostgreSQL. Memory-only mode is also available. |

Anti spam and anti scam only remove the triggering message. They do not automatically kick the member. The honeypot is the intentionally strict feature, so read [SECURITY.md](./SECURITY.md) before enabling it.

## Commands

### Application commands

| Command | Purpose |
| --- | --- |
| `/about` | Shows the current Bear Security version and project summary. |
| `/security` | Opens the Components V2 security panel. Requires Manage Server. |
| `/welcome [channel]` | Sets the welcome channel. Run without a channel to disable it. |
| `/leave [channel]` | Sets the leave channel. Run without a channel to disable it. |
| `/autorole [role]` | Sets the role given to new members. Run without a role to disable it. |

### Legacy `b!` prefix

The old project now has a small legacy prefix as a nod to the original bot style:

```text
b!ping
b!about
b!security
b!settings
```

The `b!` commands are intentionally read only. Settings changes go through Discord interactions so Bear Security can use the permission data Discord includes with the interaction.

## Discord Components V2

Bear Security uses Discord's newer Components V2 system for the security panel, notices, welcome messages, and leave messages.

Components V2 messages use Discord's `IS_COMPONENTS_V2` message flag. Traditional message content and embeds cannot be mixed into those messages, so Bear Security renders the visible text through Text Display and Container components instead.

The security panel does not use accent colors. Its controls use Discord's secondary button style, which keeps the panel visually neutral.

## Discord setup

Create an application and bot in the [Discord Developer Portal](https://discord.com/developers/applications), then copy `.env.example` to `.env` and set your token:

```env
DISCORD_TOKEN=your_bot_token
```

### Gateway intents

Enable these privileged intents in the Developer Portal:

- Server Members Intent
- Message Content Intent

Server Members is needed for welcome, leave, and autorole events. Message Content is needed for the legacy `b!` commands and for message based anti spam and anti scam checks.

### Bot permissions

Only grant permissions for features you actually use. A normal installation may need:

- View Channels
- Send Messages
- Manage Messages for anti spam and anti scam
- Manage Roles for autorole
- Manage Channels for the honeypot
- Kick Members for the honeypot

Moderators with Administrator, Manage Server, or Manage Messages are exempt from automatic message enforcement. Bots are also ignored.

## Storage

Choose a backend with `STORAGE_BACKEND`.

### Memory

Good for testing. Settings disappear when the bot restarts.

```env
STORAGE_BACKEND=memory
```

### Turso

Bear Security uses Turso's remote libSQL Rust client.

```env
STORAGE_BACKEND=turso
TURSO_DATABASE_URL=libsql://your-database.turso.io
TURSO_AUTH_TOKEN=your_token
```

### PostgreSQL

PostgreSQL connections use Rustls for TLS.

```env
STORAGE_BACKEND=postgres
DATABASE_URL=postgresql://user:password@host/database?sslmode=require
```

Bear Security creates the small `guild_settings` table automatically when using Turso or PostgreSQL.

## Running it

Bear Security targets Rust 1.89.

```bash
git clone https://github.com/mykeyy/Bear-Secuirty.git
cd Bear-Secuirty
cp .env.example .env
cargo run --release
```

On Windows PowerShell:

```powershell
Copy-Item .env.example .env
cargo run --release
```

### Nix

The repository also keeps a small Nix development shell:

```bash
nix develop
cargo run --release
```

The old Python era `flake.lock` was removed because it pinned a toolchain from before the Rust rewrite. Running `nix develop` will resolve the current flake input.

## Project structure

```text
Bear-Secuirty/
├── src/
│   ├── main.rs       # Discord gateway, commands, events, moderation actions
│   ├── security.rs   # Spam and scam detection
│   ├── storage.rs    # Memory, Turso, and PostgreSQL settings storage
│   └── ui.rs         # Discord Components V2 layouts
├── .github/
│   └── workflows/
│       └── ci.yml
├── .env.example
├── Cargo.toml
├── CHANGELOG.md
├── SECURITY.md
├── LICENSE
├── flake.nix
└── rust-toolchain.toml
```

## Legacy screenshots

These screenshots are from the original 2025 Python version. They are kept as part of the project's history and do not represent the final Rust Components V2 interface.

<table>
  <tr>
    <td align="center">
      <strong>Bot status</strong><br><br>
      <img src="./Screenshots/bot_status.png" alt="Legacy Bear Security bot status" />
    </td>
  </tr>
  <tr>
    <td align="center">
      <strong>Bear command</strong><br><br>
      <img src="./Screenshots/bear_command.png" alt="Legacy Bear command" />
    </td>
  </tr>
  <tr>
    <td align="center">
      <strong>Bot profile</strong><br><br>
      <img src="./Screenshots/bot_profile.png" alt="Legacy Bear Security profile" />
    </td>
  </tr>
</table>

## Release

`1.0.0` is the final planned release before this repository is archived. See [CHANGELOG.md](./CHANGELOG.md) for the difference between the original Python prototype and the Rust rewrite.

There is no GitHub Package for Bear Security. This is an application binary rather than a reusable Rust library, so publishing a package would add little value. Build it directly from the repository instead.

## License

Bear Security is available under the [MIT License](./LICENSE).

You are free to use, modify, and redistribute the code under the terms of that license.
