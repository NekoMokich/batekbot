import os
import random

import discord
from discord import app_commands
from discord.ext import commands


async def music_autocomplete(
    interaction: discord.Interaction, current: str
) -> list[app_commands.Choice[str]]:
    music_dir = "music"
    if not os.path.exists(music_dir):
        return []
    files = [f for f in os.listdir(music_dir) if f.endswith((".mp3", ".wav", ".ogg"))]
    matches = [f for f in files if current.lower() in f.lower()]
    return [app_commands.Choice(name=match, value=match) for match in matches[:25]]


class Music(commands.Cog):
    def __init__(self, bot):
        self.client = bot

    @app_commands.command(name="play", description="Я НАФАРМИЛ КРИПТЫ КАК СКУФ")
    @app_commands.autocomplete(filename=music_autocomplete)
    async def play(self, interaction: discord.Interaction, filename: str = None):
        await interaction.response.defer()

        if not interaction.user.voice:
            await interaction.followup.send("❌ Вы должны находиться в голосовом канале!")
            return

        channel = interaction.user.voice.channel
        voice_client = interaction.guild.voice_client

        if not voice_client:
            voice_client = await channel.connect()
        else:
            if not voice_client.is_connected():
                await voice_client.disconnect(force=True)
                voice_client = await channel.connect()
            elif voice_client.channel != channel:
                await voice_client.move_to(channel)

        music_dir = "music"

        if not os.path.exists(music_dir):
            os.makedirs(music_dir)
            await interaction.followup.send(
                f"📁 Папка '{music_dir}' не найдена. Я создал её, закинь туда музыку!"
            )
            return

        if filename is None:
            files = [f for f in os.listdir(music_dir) if f.endswith((".mp3", ".wav", ".ogg"))]
            if not files:
                await interaction.followup.send("🪹 В папке 'music' нет аудиофайлов.")
                return
            filename = random.choice(files)
        else:
            if not filename.endswith((".mp3", ".wav", ".ogg")):
                filename += ".mp3"

        file_path = os.path.join(music_dir, filename)

        if not os.path.exists(file_path):
            await interaction.followup.send(f"❌ Файл '**{filename}**' не найден.")
            return

        if voice_client.is_playing():
            voice_client.stop()

        try:
            source = discord.FFmpegPCMAudio(file_path)
            voice_client.play(source, after=lambda e: print(f"Ошибка плеера: {e}") if e else None)
            await interaction.followup.send(f"🎶 Сейчас играет: **{filename}**")
        except Exception as e:
            await interaction.followup.send(f"❌ Ошибка при воспроизведении: {e}")

    @app_commands.command(name="stop", description="ВЫКЛЮЧИ ЕГО НАХУЙ")
    async def stop(self, interaction: discord.Interaction):
        voice_client = interaction.guild.voice_client

        if voice_client and voice_client.is_playing():
            voice_client.stop()
            await interaction.response.send_message("🛑 Музыка остановлена.")
        else:
            await interaction.response.send_message("❌ Сейчас ничего не играет.")


async def setup(bot):
    await bot.add_cog(Music(bot))
