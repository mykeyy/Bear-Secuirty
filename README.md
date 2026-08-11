<div align="center">
  <img src="./Image/bear.png" alt="Bear Sec Bot logo" width="160" />

  <h1>Bear Sec Bot</h1>

  <p>A small Discord bot I built with Python, discord.py, and Nix.</p>

  <p>
    <img alt="Python" src="https://img.shields.io/badge/Python-3.12-3776AB?logo=python&logoColor=white" />
    <img alt="discord.py" src="https://img.shields.io/badge/discord.py-2.x-5865F2?logo=discord&logoColor=white" />
    <img alt="Nix" src="https://img.shields.io/badge/Nix-Flake-5277C3?logo=nixos&logoColor=white" />
    <img alt="License" src="https://img.shields.io/badge/License-MIT-green" />
    <img alt="Status" src="https://img.shields.io/badge/Status-Archived-lightgrey" />
  </p>
</div>

> [!IMPORTANT]
> This project is archived and is no longer actively maintained.

## A small note

I haven't used or worked on Bear Sec Bot in quite a while. It was a small project I made while experimenting with Discord bots, slash commands, Python, and a Nix based development setup.

I'm keeping the repository public because I still like having old projects around as a record of what I built and learned at the time.

If you somehow find this repository in the future and want to use it, feel free to fork it or take pieces from it. Just expect that some things may no longer work without changes. Discord's API, `discord.py`, Python packages, Nix packages, and bot configuration can all change over time.

## What it does

Bear Sec Bot is intentionally simple. The final version has two slash commands:

| Command | What it does |
| --- | --- |
| `/ping` | Shows the bot's Discord latency and current uptime. |
| `/bear` | Sends a small response to confirm the bot is alive. |

When the bot starts, it also sets an idle Discord presence with a custom watching activity and syncs its application commands.

## Built with

- Python
- `discord.py`
- `python-dotenv`
- Nix flakes

The included Nix development shell uses Python 3.12 and also includes tools such as Ruff, Black, Pyright, BasedPyright, and Pytest.

## Project structure

```text
Bear-Secuirty/
├── bot.py
├── flake.nix
├── flake.lock
├── requirements.txt
├── .env.example
├── Image/
└── Screenshots/
```

## Running it

> [!WARNING]
> These instructions describe how the archived version was intended to run. If you are using this much later, check the current Discord developer requirements and dependency versions first.

### Clone the repository

```bash
git clone https://github.com/mykeyy/Bear-Secuirty.git
cd Bear-Secuirty
```

### Add your Discord token

Copy the example environment file:

```bash
cp .env.example .env
```

Then add your bot token:

```env
DISCORD_TOKEN=your-bot-token-here
```

Never commit your real Discord bot token.

### Using Nix

If you have Nix with flakes enabled:

```bash
nix develop
python bot.py
```

### Using a normal Python environment

You can also use the included `requirements.txt`:

```bash
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
python bot.py
```

On Windows PowerShell:

```powershell
.\.venv\Scripts\Activate.ps1
pip install -r requirements.txt
python bot.py
```

## Discord configuration

The archived code enables the Message Content intent. If you run the project unchanged, make sure the corresponding privileged intent is enabled for your bot in the Discord Developer Portal.

The bot currently uses slash commands rather than traditional message commands, so this is also one of the first things I would review if I ever rebuilt the project.

## Screenshots

<table>
  <tr>
    <td align="center">
      <strong>Bot status</strong><br><br>
      <img src="./Screenshots/bot_status.png" alt="Bear Sec Bot status" />
    </td>
  </tr>
  <tr>
    <td align="center">
      <strong>Bear command</strong><br><br>
      <img src="./Screenshots/bear_command.png" alt="Bear command" />
    </td>
  </tr>
  <tr>
    <td align="center">
      <strong>Bot profile</strong><br><br>
      <img src="./Screenshots/bot_profile.png" alt="Bear Sec Bot profile" />
    </td>
  </tr>
</table>

## If you want to revive it

I would treat this repository as a starting point rather than something ready to deploy today. Check the current `discord.py` documentation, Discord intents and permissions, package versions, and the Nix flake before putting the bot back online.

The project is small enough that rewriting parts of it may make more sense than trying to preserve every old implementation detail.

## License

Bear Sec Bot is available under the [MIT License](./LICENSE).

You are free to use, modify, and redistribute the code under the terms of that license.
