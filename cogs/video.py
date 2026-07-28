import random
import discord
from discord import app_commands
from discord.ext import commands
import yt_dlp


def get_youtube_videos(channel_url: str, tab: str = "videos", max_count: int = None) -> list[str]:
    """
    Выгружает ID видео с указанной вкладки канала ('videos' или 'shorts') с помощью yt-dlp.
    Если max_count = None, выгружаются ВСЕ видео без каких-либо ограничений.
    """
    target_url = f"{channel_url.rstrip('/')}/{tab}"
    ydl_opts = {
        "extract_flat": "in_playlist",
        "quiet": True,
        "no_warnings": True,
        "skip_download": True,
    }

    # Если ограничение передано, добавляем его в настройки
    if max_count is not None:
        ydl_opts["playlistend"] = max_count

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

    # Кнопка 1: Выдача случайного Шортса
    @discord.ui.button(label="📱 Случайный Шортс", style=discord.ButtonStyle.blurple)
    async def btn_shorts(self, interaction: discord.Interaction, button: discord.ui.Button):
        if not self.shorts_list:
            await interaction.response.send_message("Шортсов пока нет :(", ephemeral=True)
            return

        random_id = random.choice(self.shorts_list)
        url = f"https://youtu.be/{random_id}"
        await interaction.response.send_message(f"Вот твой шортс:\n{url}")

    # Кнопка 2: Выдача случайного обычного видео
    @discord.ui.button(label="📺 Обычное видео", style=discord.ButtonStyle.green)
    async def btn_video(self, interaction: discord.Interaction, button: discord.ui.Button):
        if not self.videos_list:
            await interaction.response.send_message("Обычных видео пока нет :(", ephemeral=True)
            return

        random_id = random.choice(self.videos_list)
        url = f"https://youtu.be/{random_id}"
        await interaction.response.send_message(f"Вот обычный видос:\n{url}")


# 2. ПОЛУЧЕНИЕ ВСЕХ СПИСКОВ ЧЕРЕЗ YT-DLP (БЕЗ ОГРАНИЧЕНИЙ)
print("Загрузка абсолютно всех видео и шортсов c YouTube через yt-dlp...")

# Раис (без ограничений)
video_ids_rais = get_youtube_videos("https://www.youtube.com/@batek_official9120", "shorts")
video_ids_rais_videos = get_youtube_videos("https://www.youtube.com/@batek_official9120", "videos")

# Викторович (без ограничений)
video_ids_victorovich = get_youtube_videos("https://www.youtube.com/@batya_viktorovich", "shorts")
video_ids_victorovich_videos = get_youtube_videos("https://www.youtube.com/@batya_viktorovich", "videos")

# Тимур (без ограничений)
video_ids_timur_shorts = get_youtube_videos("https://www.youtube.com/@SravneniyaOfficial", "shorts")
video_ids_timur_video = get_youtube_videos("https://www.youtube.com/@SravneniyaOfficial", "videos")

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