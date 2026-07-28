import discord
from discord.ext import commands 
from discord import app_commands
import random
import scrapetube
import os
from dotenv import load_dotenv

load_dotenv()
main_token = os.getenv("main")
test_token = os.getenv("test")


# а тут епта бля короче у нас настройки
intents = discord.Intents.default()
intents.members = True
intents.message_content = True
bot = commands.Bot(command_prefix="!", intents=intents)


# Синхронизация слеш-команд при запуске
@bot.event
async def setup_hook():
    await bot.load_extension("cogs.music")
    await bot.load_extension("cogs.roflcal")
    await bot.load_extension("cogs.video")

@bot.event
async def on_ready():
    await bot.tree.sync()
    print(f'Бот {bot.user} запущен и слеш-команды синхронизированы!')
   
bot.run(test_token)


