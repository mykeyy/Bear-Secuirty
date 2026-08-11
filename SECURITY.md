# Security policy

Bear Security is a small Discord moderation project. It is not a replacement for Discord's own AutoMod, account security controls, or a dedicated moderation team.

## Supported version

Only the final Rust release, `1.0.x`, is considered current. The older Python implementation has been retired.

## Secrets

Never commit your Discord bot token, Turso token, or PostgreSQL connection string. Keep them in environment variables or a local `.env` file. The repository ignores `.env` by default.

If a token is exposed, rotate it immediately from the service that issued it.

## Discord permissions

Grant the bot only the permissions required by the features you use. Typical installations may need:

- View Channels and Send Messages
- Manage Messages for anti spam and anti scam cleanup
- Kick Members if the honeypot is enabled
- Manage Roles if autorole is enabled
- Manage Channels if the honeypot is enabled

The bot also uses the privileged Guild Members and Message Content gateway intents. Guild Members is used for welcome, leave, and autorole events. Message Content is required for the legacy `b!` prefix and message based spam/scam detection.

## Honeypot behavior

The honeypot is intentionally strict. When enabled, Bear Security creates a clearly labelled channel and warns members not to post there. A non-bot, non-moderator member who sends a message in that channel can be kicked automatically.

Do not enable the honeypot unless you understand that behavior. Anti spam and anti scam do not kick members. They only remove the triggering message.

## Reporting a problem

If this repository is still active, open a GitHub issue without including secrets, tokens, private database URLs, or private server data.

Once the repository is archived, treat the code as unsupported historical software. Fork it and patch it before relying on it in a production Discord server.
