import asyncio
import os
import platform
import random
import shutil
import socket
import subprocess
import sys
import time

import discord
import distro
import psutil
import uptime
from discord import app_commands
from discord.ext import commands


def generate_batekfetch_info() -> str:
    """Генерация системной информации без блокировки асинхронного потока бота."""
    if sys.platform == "win32":
        try:
            sys.stdout.reconfigure(encoding="utf-8")
        except Exception:
            pass

    # 1. Получение названия процессора
    if os.path.exists("/proc/cpuinfo"):
        try:
            command = 'cat /proc/cpuinfo | grep "model name" | cut -f 2 -d ":"'
            result = os.popen(command).read().strip()
            cpu_name = result.split("\n")[0].strip() if result else platform.processor()
        except Exception:
            cpu_name = platform.processor() or "Unknown CPU"
    else:
        cpu_name = platform.processor() or "Unknown CPU"

    # 2. Uptime
    try:
        seconds_uptime = uptime.uptime()
    except Exception:
        seconds_uptime = time.time() - psutil.boot_time()

    days = int(seconds_uptime // 86400)
    hours = int((seconds_uptime % 86400) // 3600)
    minutes = int((seconds_uptime % 3600) // 60)

    # 3. Пакетные менеджеры
    package_managers = {
        "dpkg": "dpkg-query -f '${binary:Package}\n' -W",
        "rpm": "rpm -qa",
        "pacman": "pacman -Q",
        "apk": "apk info",
        "dnf": "dnf list installed",
        "emerge": "qlist -I",
        "zypper": "zypper se --installed-only",
        "xbps-query": "xbps-query -l",
    }

    for manager, command in package_managers.items():
        if shutil.which(manager):
            try:
                subprocess.check_output(command, shell=True, text=True)
                break
            except Exception:
                pass

    # 4. Память
    ram_bytes = psutil.virtual_memory().total
    ram_gygabytes = round(ram_bytes / (1024**3), 2)
    ram_used_gb = round(psutil.virtual_memory().used / (1024**3), 2)

    # 5. Диск
    disk = shutil.disk_usage("/")
    used_gb = disk.used / (1024**3)
    total_gb = disk.total / (1024**3)

    uptime_str = (
        f"{minutes}m"
        if days == 0 and hours == 0
        else f"{hours}h, {minutes}m"
        if days == 0
        else f"{days}d, {hours}h, {minutes}m"
    )

    return (
        "```text\n"
        " _________    \n"
        "|______  /    OS:      " + (distro.name() or platform.system()) + "\n"
        "      / /     HOST:    " + socket.gethostname() + "\n"
        "     / /      CPU:     " + cpu_name + "\n"
        "    / /       UPTIME:  " + uptime_str + "\n"
        "   / /        DISK:    " + f"{used_gb:.2f} GB / {total_gb:.2f} GB" + "\n"
        "  / /_____    RAM:     " + f"{ram_used_gb:.2f} GB / {ram_gygabytes:.2f} GB" + "\n"
        " /________|   \n"
        "```"
    )


class RoflCal(commands.Cog):
    def __init__(self, bot):
        self.client = bot

    @app_commands.command(name="grandmobile", description="грэндмобайл")
    async def granmobile(self, interaction: discord.Interaction):
        await interaction.response.send_message(
            'гранд мобайл это игра с открытым миром где можно стать кем угодно хоть блогером полицейским или боссом мафии. А скачать ее можно в апп строке или гугл плей или по ссылке в шапке профиля, а по промокоду "БАТЕК" вы получите крутые бонусы'
        )

    @app_commands.command(name="bomjara", description="АКАКФИЛЬМНАНОУТБУКЕВКЛЮЧИТЬ")
    async def bomjara(self, interaction: discord.Interaction):
        aiditinaxuy = random.randint(0, 10)
        BASE_DIR = os.path.dirname(os.path.abspath(__file__))

        if aiditinaxuy > 5:
            file_path = os.path.join(BASE_DIR, "..", "video", "bomjara1.mp4")
            await interaction.response.send_message(file=discord.File(fp=file_path))
        else:
            file_path = os.path.join(BASE_DIR, "..", "video", "bomjaro2.mp4")
            await interaction.response.send_message(file=discord.File(fp=file_path))

    @app_commands.command(
        name="roflvideo", description="не смешная хуйня про 320 пошути же даун ебучий"
    )
    async def roflvideo(self, interaction: discord.Interaction):
        aiditinaxuy = random.randint(0, 10)
        BASE_DIR = os.path.dirname(os.path.abspath(__file__))

        if aiditinaxuy >= 5:
            file_path = os.path.join(BASE_DIR, "..", "video", "makuos.mp4")
            await interaction.response.send_message(file=discord.File(fp=file_path))
        else:
            file_path = os.path.join(BASE_DIR, "..", "video", "shefos.mp4")
            await interaction.response.send_message(file=discord.File(fp=file_path))

    @app_commands.command(name="help", description="слєш хелп")
    async def help_command(self, interaction: discord.Interaction):
        await interaction.response.send_message("аидитынахуй")

    @app_commands.command(name="kupyansk", description="купянскнаш")
    async def kupyansk(self, interaction: discord.Interaction):
        BASE_DIR = os.path.dirname(os.path.abspath(__file__))
        file_path = os.path.join(BASE_DIR, "..", "video", "kupyanskdis.mp4")
        await interaction.response.send_message(file=discord.File(fp=file_path))

    @app_commands.command(name="batekfetch", description="ЭЭЭФЕТЧИ")
    async def batekfetch(self, interaction: discord.Interaction):
        # Выполняем системные проверки асинхронно в отдельном потоке
        output_msg = await asyncio.to_thread(generate_batekfetch_info)
        await interaction.response.send_message(output_msg)


async def setup(bot):
    await bot.add_cog(RoflCal(bot))
