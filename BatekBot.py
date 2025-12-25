import nextcord
from nextcord.ext import commands
import random
import scrapetube

videos = scrapetube.get_channel(channel_url="https://www.youtube.com/@batek_official9120", content_type='shorts')
serverID = 1058917331021090887 #поменять если сервак другой (бублики ненадо)
intents = nextcord.Intents.default()
intents.members = True
intents.message_content = True

bot = commands.Bot()
video_ids = [video['videoId'] for video in videos]

if not video_ids:
    print("Аидитынахуй")

@bot.slash_command(name="batek", description="белые розы", guild_ids=[serverID])
async def batek(interaction: nextcord.Interaction):

    aiditinaxuy = random.randint(1,100)

    if (aiditinaxuy > 10):
        random_id = random.choice(video_ids)
        await interaction.response.send_message("https://www.youtube.com/shorts/" + random_id)
    else:
        await interaction.response.send_message("аидитынахуй")

@bot.slash_command(name="help", description="слєш хелп", guild_ids=[serverID])
async def help(interaction: nextcord.Interaction):
    await interaction.response.send_message("аидитынахуй")

@bot.slash_command(name="grandmobile", description="грэндмобайл", guild_ids=[serverID])
async def granmobile(interaction: nextcord.Interaction):
    await interaction.response.send_message('гранд мобайл это игра с открытым миром где можно стать кем угодно хоть блогером полицейским или боссом мафии. А скачать ее можно в апп строке или гугл плей или по ссылке в шапке профиля, а по промокоду "БАТЕК" вы получите крутые бонусы')


bot.run("MTQ1MzUzNzExNjQ1MjU1Mjc3MQ.G5fUXK.8nM8IwZ9rIPLDfEvUQHLQVug4MJtJRNz6gOM-A") #поменять если другой бот (у нас батек так что нахуй)