from pytubefix import YouTube
import nextcord
from nextcord.ext import commands
import random
import scrapetube
import os
import distro
import socket
import os
import psutil
from uptime import uptime
import subprocess
import shutil
import platform

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
    def init(self, video_id):
        super().init()
        self.value = None
        self.video_id = video_id

    @nextcord.ui.button(label = "СКАЧАТЬ БЕЗ ВИРУСОВ (ЕСЛИ НЕТ ВИДЕО КНОПКУ ДРОЧИТЕ!!!)", style = nextcord.ButtonStyle.green)
    async def donwload(self, button: nextcord.ui.Button, interaction: nextcord.Interaction):

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

#фетч
@bot.slash_command(name="fastfetch", description="ЭЭЭФЕТЧИ")
async def fastfetch(interaction: nextcord.Interaction):
    # processor
    command = 'cat /proc/cpuinfo | grep "model name" | cut -f 2 -d ":"'
    result = os.popen(command).read().strip()

# uptime block
    seconds_uptime = uptime()
    days = int(seconds_uptime // 86400)
    hours = int((seconds_uptime % 86400) // 3600)
    minutes = int((seconds_uptime % 3600) // 60)

    ##package
    package_managers = {
        "dpkg": "dpkg-query -f '${binary:Package}\n' -W",  # Debian, Ubuntu, Kali, Mint
        "rpm": "rpm -qa",  # RHEL, CentOS, Fedora, OpenSUSE
        "pacman": "pacman -Q",  # Arch Linux, Manjaro
        "apk": "apk info",  # Alpine Linux
        "dnf": "dnf list installed",  # Modern RHEL/Fedora (fallback if rpm missing)
        "emerge": "qlist -I",  # Gentoo
        "zypper": "zypper se --installed-only",  # OpenSUSE (fallback if rpm missing)
        "xbps-query": "xbps-query -l"  # Void Linux
    }

    pckg_count = 0

    for manager, command in package_managers.items():
        if shutil.which(manager):
            output = subprocess.check_output(command, shell=True, text=True)
            lines = [line for line in output.splitlines() if line.strip()]
            count = len(lines)
            pckg_count = count

    ##ram
    # 1. Исправляем математику: добавляем ** (возведение в степень)
    ram_bytes = psutil.virtual_memory().total
    ram_gygabytes = round(ram_bytes / (1024 ** 3), 2)
    used_gb = round(psutil.virtual_memory().used / (1024 ** 3), 2)

    # 2. Исправляем f-строку: выносим операцию с '\n' в отдельную переменную
    cpu_name = result.split('\n')[0].strip()

    await interaction.response.send_message(f"os: {distro.name()} \n"
                                            f"host: {socket.gethostname()} \n"
                                            f"cpu: {cpu_name} \n"  # Теперь здесь нет бэкслеша
                                            f"uptime: {f'{minutes}m' if days == 0 and hours == 0 else f'{hours}h, {minutes}m' if days == 0 else f'{days}d, {hours}h, {minutes}m'} \n"
                                            f"pkgs: {pckg_count} \n"
                                            f"ram: {used_gb}gb / {ram_gygabytes}gb \n", ephemeral=True)

bot.run("MTAyMjkzNzM4NDYwNzgxMzcyMg.G3UxKT.GqTJsRtjwOIe-N0lJnjy6BHaRiOOXoobt-iYnA") #поменять если другой бот (у нас батек так что нахуй)