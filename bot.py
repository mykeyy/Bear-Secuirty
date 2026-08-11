from __future__ import annotations

import os
import time

import discord
from discord import app_commands
from dotenv import load_dotenv

load_dotenv()
TOKEN = os.getenv("DISCORD_TOKEN")


class BearSecClient(discord.Client):
    def __init__(self) -> None:
        super().__init__(
            intents=discord.Intents.none(),
            status=discord.Status.idle,
            activity=discord.Activity(
                type=discord.ActivityType.watching,
                name="Yua's Cove",
            ),
        )
        self.tree = app_commands.CommandTree(self)
        self.started_at = time.monotonic()

    async def setup_hook(self) -> None:
        synced = await self.tree.sync()
        print(f"Synced {len(synced)} application command(s).")

    async def on_ready(self) -> None:
        if self.user is not None:
            print(f"Logged in as {self.user} (ID: {self.user.id})")


client = BearSecClient()


@client.tree.command(
    name="ping",
    description="Check the bot's latency and uptime.",
)
async def ping(interaction: discord.Interaction) -> None:
    latency_ms = round(client.latency * 1000)
    uptime_seconds = int(time.monotonic() - client.started_at)

    days, remainder = divmod(uptime_seconds, 86_400)
    hours, remainder = divmod(remainder, 3_600)
    minutes, seconds = divmod(remainder, 60)

    uptime_parts = []
    if days:
        uptime_parts.append(f"{days}d")
    if hours or days:
        uptime_parts.append(f"{hours}h")
    if minutes or hours or days:
        uptime_parts.append(f"{minutes}m")
    uptime_parts.append(f"{seconds}s")

    embed = discord.Embed(
        title="Ping Pong!",
        description="The bot is online and responding.",
        color=discord.Color.blurple(),
    )
    embed.add_field(name="Latency", value=f"{latency_ms} ms", inline=True)
    embed.add_field(name="Uptime", value=" ".join(uptime_parts), inline=True)
    embed.set_footer(text="Bear Sec Bot | Powered by Yua's Cove")

    await interaction.response.send_message(embed=embed)


@client.tree.command(
    name="bear",
    description="Check whether Bear Sec Bot is online.",
)
async def bear(interaction: discord.Interaction) -> None:
    await interaction.response.send_message("Hi, I am working!")


if not TOKEN:
    raise SystemExit("DISCORD_TOKEN is not set. Add it to your environment or .env file.")

client.run(TOKEN)
