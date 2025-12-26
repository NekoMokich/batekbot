from pytubefix import YouTube
import nextcord
from nextcord.ext import commands
import random
import scrapetube
import os

#а тут епта бля короче у нас настройки
intents = nextcord.Intents.default()
intents.members = True
intents.message_content = True
bot = commands.Bot(command_prefix="!", intents=intents)

#блок с раиской
videos_rais = scrapetube.get_channel(channel_url="https://www.youtube.com/@batek_official9120", content_type='shorts')
video_ids_rais = [video['videoId'] for video in videos_rais]
if not video_ids_rais:
    print("Аидитынахуй")

# блок с викторовичем
videos_victorovich = scrapetube.get_channel(channel_url="https://www.youtube.com/@batya_viktorovich", content_type='shorts')
video_ids_victorovich = [video['videoId'] for video in videos_victorovich]
if not video_ids_victorovich:
    print("Аидитынахуй")

#кнопки с вирусами
class Donwload(nextcord.ui.View):
    def __init__(self, video_id):
        super().__init__()
        self.value = None
        self.video_id = video_id

    @nextcord.ui.button(label = "СКАЧАТЬ БЕЗ ВИРУСОВ (ЕСЛИ НЕТ ВИДЕО КНОПКУ ДРОЧИТЕ!!!)", style = nextcord.ButtonStyle.green)
    async def donwload(self, button: nextcord.ui.Button, interaction: nextcord.Interaction):
        from pytubefix import YouTube

        url = f"https://www.youtube.com/shorts/{self.video_id}"
        yt = YouTube(url)

        stream = yt.streams.get_highest_resolution() #сами вирусы
        video_path = stream.download()

        await interaction.response.send_message("вот твои ебучие вирусы", file=nextcord.File(video_path))
        os.remove(video_path)

        self.value = True
        self.stop()

#блок с батьком
@bot.slash_command(name="batek", description="белые розы")
async def batek(interaction: nextcord.Interaction):
    aiditinaxuy = random.randint(1,100)

    if aiditinaxuy > 10:
        random_id = random.choice(video_ids_rais)
        view = Donwload(random_id)
        # Передаём view в параметре view
        await interaction.response.send_message(
            "https://www.youtube.com/shorts/" + random_id,
            view=view
        )
        await view.wait()
    else:
        await interaction.response.send_message("аидитынахуй")

#блок с викторовичем
@bot.slash_command(name="victorovich", description="В СТАРОМ ПАРКЕ ПАХНЕТ ХВОЙНОЙ ТИШИНОЙ")
async def victorovich(interaction: nextcord.Interaction):
    aiditinaxuy = random.randint(1,100)

    if aiditinaxuy > 10:
        random_id = random.choice(video_ids_victorovich)
        view = Donwload(random_id)
        # Передаём view в параметре view
        await interaction.response.send_message(
            "https://www.youtube.com/shorts/" + random_id,
            view=view
        )
        await view.wait()
    else:
        await interaction.response.send_message("аидитынахуй")

@bot.slash_command(name="help", description="слєш хелп")
async def help(interaction: nextcord.Interaction):
    await interaction.response.send_message("аидитынахуй")

@bot.slash_command(name="bomjara", description="АКАКФИЛЬМНАНОУТБУКЕВКЛЮЧИТЬ") #БОМЖАРО
async def bomjara(interaction: nextcord.Interaction):
    file_path = "video_2025-11-10_11-48-39.mp4"
    await interaction.response.send_message("АКАКФИЛЬМ НА НОУТБУКЕ ВКЛЮЧИТ", file=nextcord.File(fp=file_path))

@bot.slash_command(name="grandmobile", description="грэндмобайл")
async def granmobile(interaction: nextcord.Interaction):
    await interaction.response.send_message('гранд мобайл это игра с открытым миром где можно стать кем угодно хоть блогером полицейским или боссом мафии. А скачать ее можно в апп строке или гугл плей или по ссылке в шапке профиля, а по промокоду "БАТЕК" вы получите крутые бонусы')

bot.run("MTQ1MzUzNzExNjQ1MjU1Mjc3MQ.G5fUXK.8nM8IwZ9rIPLDfEvUQHLQVug4MJtJRNz6gOM-A") #поменять если другой бот (у нас батек так что нахуй)