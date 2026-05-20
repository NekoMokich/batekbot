import discord
from discord.ext import commands 
from discord import app_commands
import random
import scrapetube
import os

# а тут епта бля короче у нас настройки
intents = discord.Intents.default()
intents.members = True
intents.message_content = True
bot = commands.Bot(command_prefix="!", intents=intents)


# Синхронизация слеш-команд при запуске
@bot.event
async def on_ready():
    await bot.tree.sync()
    print(f'Бот {bot.user} запущен и слеш-команды синхронизированы!')


# блок с раиской (редиска)
videos_rais = scrapetube.get_channel(channel_url="https://www.youtube.com/@batek_official9120", content_type='shorts')
video_ids_rais = [video['videoId'] for video in videos_rais]
if not video_ids_rais:
    print("Аидитынахуй")
    
videos_rais_videos = scrapetube.get_channel(channel_url="https://www.youtube.com/@batek_official9120", content_type='videos')
video_ids_rais_videos = [video['videoId'] for video in videos_rais_videos]
if not video_ids_rais_videos:
    print("Аидитынахуй")

# блок с викторовичем
videos_victorovich = scrapetube.get_channel(channel_url="https://www.youtube.com/@batya_viktorovich",content_type='shorts')
video_ids_victorovich = [video['videoId'] for video in videos_victorovich]
if not video_ids_victorovich:
    print("Аидитынахуй")

videos_victorovich_videos = scrapetube.get_channel(channel_url="https://www.youtube.com/@batya_viktorovich", content_type='videos')
video_ids_victorovich_videos = [video['videoId'] for video in videos_victorovich_videos]    
if not video_ids_victorovich_videos:
    print("Аидитынахуй")

#Тимурка
videos_timur_shorts = scrapetube.get_channel(channel_url="https://www.youtube.com/@SravneniyaOfficial",content_type='shorts')
video_ids_timur_shorts = [video['videoId'] for video in videos_timur_shorts]
if not video_ids_timur_shorts:
    print("Аидитынахуй")

videos_timur_video = scrapetube.get_channel(channel_url="https://www.youtube.com/@SravneniyaOfficial",content_type='videos')
video_ids_timur_video = [video['videoId'] for video in videos_timur_video]
if not video_ids_timur_video:
    print("Аидитынахуй")

#скачка видео
class VideoChoice(discord.ui.View):
    # При создании кнопок передаем списки шортсов и видео
    def __init__(self, shorts_list, videos_list):
        super().__init__()
        self.shorts_list = shorts_list
        self.videos_list = videos_list

    # Кнопка для шортсов (синяя)
    @discord.ui.button(label="📱 Случайный Шортс", style=discord.ButtonStyle.blurple)
    async def btn_shorts(self, interaction: discord.Interaction, button: discord.ui.Button):
        if not self.shorts_list:
            await interaction.response.send_message("Шортсов пока нет :(", ephemeral=True)
            return
            
        random_id = random.choice(self.shorts_list)
        url = f"https://youtu.be/{random_id}"
        
        await interaction.response.send_message(f"Вот твой шортс:\n{url}")

    # Кнопка для обычных видео (зеленая)
    @discord.ui.button(label="📺 Обычное видео", style=discord.ButtonStyle.green)
    async def btn_video(self, interaction: discord.Interaction, button: discord.ui.Button):
        if not self.videos_list:
            await interaction.response.send_message("Обычных видео пока нет :(", ephemeral=True)
            return
            
        random_id = random.choice(self.videos_list)
        url = f"https://youtu.be/{random_id}"
        
        # Отправляем видео и прикрепляем твою кнопку скачивания
        await interaction.response.send_message(f"Вот обычный видос:\n{url}")

# блок с Тимуром
@bot.tree.command(name="timur", description="ЗАЧЕМ Я ПОМЕНЯЛ ЕЕ НА ЭТО НА ЧЕРНОБЕЛОЕ")
async def timur(interaction: discord.Interaction):
    aiditinaxuy = random.randint(1, 100)
    if aiditinaxuy > 10:
        view = VideoChoice(video_ids_timur_shorts, video_ids_timur_video)
        
        await interaction.response.send_message("че смотреть будем", view=view)
    else:
        await interaction.response.send_message("аидитынахуй")

# блок с батьком (Раис)
@bot.tree.command(name="batek", description="белые розы")
async def batek(interaction: discord.Interaction):
    aiditinaxuy = random.randint(1, 100)

    if aiditinaxuy > 10:
        # Передаем списки Раиса (шортсы и обычные видео) в наш класс с кнопками выбора
        view = VideoChoice(video_ids_rais, video_ids_rais_videos)
        await interaction.response.send_message("че смотреть будем", view=view)
    else:
        await interaction.response.send_message("аидитынахуй")

# блок с викторовичем
@bot.tree.command(name="victorovich", description="В СТАРОМ ПАРКЕ ПАХНЕТ ХВОЙНОЙ ТИШИНОЙ")
async def victorovich(interaction: discord.Interaction):
    aiditinaxuy = random.randint(1, 100)

    if aiditinaxuy > 10:
        view = VideoChoice(video_ids_victorovich, video_ids_victorovich_videos)
        await interaction.response.send_message("че смотреть будем", view=view)
    else:
        await interaction.response.send_message("аидитынахуй")


@bot.tree.command(name="help", description="слєш хелп")
async def help_command(
interaction: discord.Interaction):  # Заменил имя функции help -> help_command во избежание конфликта со встроенным хелпом
    await interaction.response.send_message("аидитынахуй")

@bot.tree.command(name="kupyansk", description="купянскнаш")  # КУПЯНСК
async def kupyansk(interaction: discord.Interaction):
    BASE_DIR = os.path.dirname(os.path.abspath(__file__))
    file_path = os.path.join(BASE_DIR, "video", "kupyanskdis.mp4")
    await interaction.response.send_message(file=discord.File(fp=file_path))


@bot.tree.command(name="bomjara", description="АКАКФИЛЬМНАНОУТБУКЕВКЛЮЧИТЬ")  # БОМЖАРО(лучший дистр)
async def bomjara(interaction: discord.Interaction):
    aiditinaxuy = random.randint(0, 10)
    BASE_DIR = os.path.dirname(os.path.abspath(__file__))
    
    if aiditinaxuy > 5:
        file_path = os.path.join(BASE_DIR, "video", "bomjara1.mp4")
        await interaction.response.send_message(file=discord.File(fp=file_path))
    else:
        file_path = os.path.join(BASE_DIR, "video", "bomjaro2.mp4")
        await interaction.response.send_message(file=discord.File(fp=file_path))

@bot.tree.command(name="roflvideo", description="не смешная хуйня про 320 пошути же даун ебучий")  # другое
async def roflvideo(interaction: discord.Interaction):
    aiditinaxuy = random.randint(0, 10)
    BASE_DIR = os.path.dirname(os.path.abspath(__file__))

    if aiditinaxuy >= 5:
        file_path = os.path.join(BASE_DIR, "video", "makuos.mp4")
        await interaction.response.send_message(file=discord.File(fp=file_path))
    else:
        file_path = os.path.join(BASE_DIR, "video", "shefos.mp4")
        await interaction.response.send_message(file=discord.File(fp=file_path))
        
@bot.tree.command(name="grandmobile", description="грэндмобайл")
async def granmobile(interaction: discord.Interaction):
    await interaction.response.send_message(
        'гранд мобайл это игра с открытым миром где можно стать кем угодно хоть блогером полицейским или боссом мафии. А скачать ее можно в апп строке или гугл плей или по ссылке в шапке профиля, а по промокоду "БАТЕК" вы получите крутые бонусы')


#тун тун тун сахур бр бр бр патапим
async def music_autocomplete(interaction: discord.Interaction, current: str) -> list[app_commands.Choice[str]]:
    music_dir = 'music'
    if not os.path.exists(music_dir):
        return []
    files = [f for f in os.listdir(music_dir) if f.endswith(('.mp3', '.wav', '.ogg'))]
    matches = [f for f in files if current.lower() in f.lower()]
    return [app_commands.Choice(name=match, value=match) for match in matches[:25]]

@bot.tree.command(name='play', description='Я НАФАРМИЛ КРИПТЫ КАК СКУФ')
@app_commands.autocomplete(filename=music_autocomplete)
async def play(interaction: discord.Interaction, filename: str = None):
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

    music_dir = 'music'

    if not os.path.exists(music_dir):
        os.makedirs(music_dir)
        await interaction.followup.send(f"📁 Папка '{music_dir}' не найдена. Я создал её, закинь туда музыку!")
        return

    if filename is None:
        files = [f for f in os.listdir(music_dir) if f.endswith(('.mp3', '.wav', '.ogg'))]
        if not files:
            await interaction.followup.send("🪹 В папке 'music' нет аудиофайлов.")
            return
        filename = random.choice(files)
    else:
        if not filename.endswith(('.mp3', '.wav', '.ogg')):
            filename += '.mp3'

    file_path = os.path.join(music_dir, filename)

    if not os.path.exists(file_path):
        await interaction.followup.send(f"❌ Файл '**{filename}**' не найден.")
        return

    if voice_client.is_playing():
        voice_client.stop()

    try:
        source = discord.FFmpegPCMAudio(file_path)
        voice_client.play(source, after=lambda e: print(f'Ошибка плеера: {e}') if e else None)
        await interaction.followup.send(f"🎶 Сейчас играет: **{filename}**")
    except Exception as e:
        await interaction.followup.send(f"❌ Ошибка при воспроизведении: {e}")

@bot.tree.command(name='stop', description='ВЫКЛЮЧИ ЕГО НАХУЙ')
async def stop(interaction: discord.Interaction):
    voice_client = interaction.guild.voice_client
    
    if voice_client and voice_client.is_playing():
        voice_client.stop()
        await interaction.response.send_message("🛑 Музыка остановлена.")
    else:
        await interaction.response.send_message("❌ Сейчас ничего не играет.")

bot.run("MTQ1MzUzNzExNjQ1MjU1Mjc3MQ.G5fUXK.8nM8IwZ9rIPLDfEvUQHLQVug4MJtJRNz6gOM-A")

#тут ржака какая то
