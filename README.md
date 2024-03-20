# Serenity Discord Wordle Leaderboart Bot with Shuttle

This bot will pick up on when someone posts their wordle score, e.g:

Wordle 1,005 5/6

⬜⬜⬜🟨🟩 <br>
🟩⬜⬜⬜🟩 <br>
🟩⬜🟩🟩🟩 <br>
🟩🟩🟩🟩🟩 <br>

and will save it to the leaderboard.

Someone in the discord server can then type "!wordle" and the bot will post a message with the current wordle leaderboard.

The score is calculated by averaging all of the wordle scores a user has posted. The lower the score, the better.

For more information please refer to the [Discord docs](https://discord.com/developers/docs/getting-started) as well as the [Serenity repo](https://github.com/serenity-rs/serenity) for more examples.
