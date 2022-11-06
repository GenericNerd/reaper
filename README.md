<h1 align="center" style="font-size: 48px;">Reaper</h1>
<div align="center">
A Discord bot designed to make your users reap what they sow.
</div>
<div align="center">

[![Version](https://img.shields.io/badge/version-0.1.0--dev-red)](https://img.shields.io/badge/version-0.1.0--dev-red)
</div>

## What is Reaper?
Reaper is a Discord bot that allows you to automate and make tasks easier that would normally take a lot of time and effort. It is designed to be easy to use and extremely customizable.

# Self Hosting Guide
## Requirements
- [A Discord Bot Application](https://discord.com/developers/applications)
- [Docker](https://www.docker.com/)
- **Optional:** [Docker Compose](https://docs.docker.com/compose/)

## Installation
### Docker
The prefered way to run Reaper is using Docker. This allows you to run Reaper in a container and allows you to easily update it. To run Reaper using Docker, you will need to create a `docker-compose.yml` file.
There is an example provided with the `docker-compose.example.yml` file. You can copy this file and rename it to `docker-compose.yml`. You will need to edit the file to add your bot token, the prefix you want to use and the credentials for the mongo database.
#### Linux
1. Ensure you have installed Docker (and Docker Compose if following this guide)
2. Download the Reaper Repository (either by cloning it or downloading the repository)
```bash
git clone https://github.com/GenericNerd/reaper
```
3. Enter the directory and edit your `docker-compose.yml` file to preference.
4. Run the following command to start the bot
```bash
docker-compose up --build -d
```
#### Windows
To Be Written (Linux FTW)

# License
MIT License

Copyright © 2022 Fabio Almeida