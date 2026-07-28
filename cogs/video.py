import random
import discord
from discord import app_commands
from discord.ext import commands
import yt_dlp
from concurrent.futures import ThreadPoolExecutor


def get_youtube_videos(channel_url: str, tab: str = "videos", max_count: int = 50) -> list[str]:
    """
    Выгружает ID видео с указанной вкладки канала ('videos' или 'shorts') с помощью yt-dlp.
    """
    target_url = f"{channel_url.rstrip('/')}/{tab}"
    ydl_opts = {
        "extract_flat": "in_playlist",
        "playlistend": max_count,
        "quiet": True,
        "no_warnings": True,
        "skip_download": True,
    }

    video_ids = []
    try:
        with yt_dlp.YoutubeDL(ydl_opts) as ydl:
            info = ydl.extract_info(target_url, download=False)
            if info and "entries" in info:
                for entry in info["entries"]:
                    if entry and entry.get("id"):
                        video_ids.append(entry["id"])
    except Exception as e:
        print(f"Ошибка при выгрузке {target_url}: {e}")

    return video_ids


# 1. КЛАСС КНОПОК
class VideoChoice(discord.ui.View):
    def __init__(self, shorts_list, videos_list):
        super().__init__()
        self.shorts_list = shorts_list
        self.videos_list = videos_list

    @discord.ui.button(label="📱 Случайный Шортс", style=discord.ButtonStyle.blurple)
    async def btn_shorts(self, interaction: discord.Interaction, button: discord.ui.Button):
        if not self.shorts_list:
            await interaction.response.send_message("Шортсов пока нет :(", ephemeral=True)
            return

        random_id = random.choice(self.shorts_list)
        url = f"https://youtu.be/{random_id}"
        await interaction.response.send_message(f"Вот твой шортс:\n{url}")

    @discord.ui.button(label="📺 Обычное видео", style=discord.ButtonStyle.green)
    async def btn_video(self, interaction: discord.Interaction, button: discord.ui.Button):
        if not self.videos_list:
            await interaction.response.send_message("Обычных видео пока нет :(", ephemeral=True)
            return

        random_id = random.choice(self.videos_list)
        url = f"https://youtu.be/{random_id}"
        await interaction.response.send_message(f"Вот обычный видос:\n{url}")


# 2. ПАРАЛЛЕЛЬНАЯ ЗАГРУЗКА ВСЕХ 6 КАНАЛОВ ОДНОВРЕМЕННО
print("Параллельная загрузка видео и шортсов c YouTube...")

tasks = [
    ("https://www.youtube.com/@batek_official9120", "shorts", 50),
    ("https://www.youtube.com/@batek_official9120", "videos", 50),
    ("https://www.youtube.com/@batya_viktorovich", "shorts", 50),
    ("https://www.youtube.com/@batya_viktorovich", "videos", 50),
    ("https://www.youtube.com/@SravneniyaOfficial", "shorts", 50),
    ("https://www.youtube.com/@SravneniyaOfficial", "videos", 50),
]

# Используем многопоточность для параллельного выполнения сетевых запросов
with ThreadPoolExecutor(max_workers=6) as executor:
    results = list(executor.map(lambda args: get_youtube_videos(*args), tasks))

(
    video_ids_rais,
    video_ids_rais_videos,
    video_ids_victorovich,
    video_ids_victorovich_videos,
    video_ids_timur_shorts,
    video_ids_timur_video,
) = results

print(f"Раис: shorts={len(video_ids_rais)}, videos={len(video_ids_rais_videos)}")
print(f"Викторович: shorts={len(video_ids_victorovich)}, videos={len(video_ids_victorovich_videos)}")
print(f"Тимур: shorts={len(video_ids_timur_shorts)}, videos={len(video_ids_timur_video)}")


# 3. COG КЛАСС С КОМАНДАМИ
class Video(commands.Cog):
    def __init__(self, bot):
        self.bot = bot

    @app_commands.command(name="timur", description="ЗАЧЕМ Я ПОМЕНЯЛ ЕЕ НА ЭТО НА ЧЕРНОБЕЛОЕ")
    async def timur(self, interaction: discord.Interaction):
        aiditinaxuy = random.randint(1, 100)
        if aiditinaxuy > 10:
            view = VideoChoice(video_ids_timur_shorts, video_ids_timur_video)
            await interaction.response.send_message("че смотреть будем", view=view)
        else:
            await interaction.response.send_message("аидитынахуй")

    @app_commands.command(name="batek", description="белые розы")
    async def batek(self, interaction: discord.Interaction):
        aiditinaxuy = random.randint(1, 100)
        if aiditinaxuy > 10:
            view = VideoChoice(video_ids_rais, video_ids_rais_videos)
            await interaction.response.send_message("че смотреть будем", view=view)
        else:
            await interaction.response.send_message("аидитынахуй")

    @app_commands.command(name="victorovich", description="В СТАРОМ ПАРКЕ ПАХНЕТ ХВОЙНОЙ ТИШИНОЙ")
    async def victorovich(self, interaction: discord.Interaction):
        aiditinaxuy = random.randint(1, 100)
        if aiditinaxuy > 10:
            view = VideoChoice(video_ids_victorovich, video_ids_victorovich_videos)
            await interaction.response.send_message("че смотреть будем", view=view)
        else:
            await interaction.response.send_message("аидитынахуй")


async def setup(bot):
    await bot.add_cog(Video(bot))