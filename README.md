# discord_url_sender

A fork of [discord_message_sender](https://github.com/okawak/discord_message_sender) that only clips URLs. It is intended for a private, single-user Discord channel where every message is a link.

## Overview

This is an Obsidian plugin that clips web pages you post in a Discord channel and automatically syncs them into Obsidian as Markdown.

**Key Features:**

- Automatically clips the web page at any URL posted in the channel and saves it as Markdown — no command prefix needed
- Silently ignores any message that doesn't contain a URL; nothing is saved and no notice is shown
- Syncs multiple Discord channels into a single shared clippings folder
- Lets you disable or customize the Discord notification messages sent after sync
- Can be triggered on Obsidian desktop startup or via the command palette

## Usage Flow

1. **Prepare Your Discord Environment**
    - Create a dedicated Discord server for Obsidian integration
    - Create a bot and invite it to your server
    - Specify one or more integration channels (using their channel IDs)

2. **Message Processing**
    - When you launch Obsidian, the plugin fetches messages from Discord via the API
    - Any message containing a URL → the linked page is fetched, converted to Markdown, and saved
    - Any message without a URL → ignored entirely
    - After processing, a completion notification is optionally sent to Discord

## ⚠️ Notes

- **Security:** Since this uses the Discord API, avoid sending sensitive or confidential information.
- **Supported Environment:** Requires Obsidian 1.13.0 or later and only works on desktop.

## Setup Guide

### 1. Create a Discord Bot

1. Go to the [Discord Developer Portal](https://discord.com/developers/applications)
2. Click **New Application** to create a new app
   ![image](https://d1fhrovvkiovx5.cloudfront.net/642c9b33b0d8250e770448b88d78e2c2.png)
3. **Bot Settings**
    - Select **Bot** from the left menu
    - Enable **Message Content Intent**
      ![image](https://d1fhrovvkiovx5.cloudfront.net/d284d81647f3dbf52a040cc7a6aa1362.png)
    - **Save the bot token** (⚠️ Important: Keep it secure)

### 2. Invite the Bot to Your Server

1. Go to **OAuth2** → **OAuth2 URL Generator** in the left menu
   ![image](https://d1fhrovvkiovx5.cloudfront.net/02355b8d6747734b75ae7b9799203132.png)
2. Under **Scopes**, select `bot`
3. Under **Bot Permissions**, enable:
    - View Channels
    - Send Messages (only required when sync notifications are enabled)
    - Read Message History
4. Use the generated URL to invite your bot

### 3. Get Channel IDs

1. In Discord settings, enable **Developer Mode** (in Advanced settings)
2. Right-click each channel you want to sync → **Copy Channel ID**

### 4. Required Plugin Settings

Please enter the following information in the plugin settings:

- **Bot Token**
- **Channels**: Add each Discord channel ID. A channel name is optional and only appears in Discord sync notifications (`{channelName}`); it does not affect where clippings are saved. Channel names must be unique among your configured channels.
- **Send sync notifications**: Disable this to prevent the plugin from posting completion messages to Discord.
- **Notification templates**: Optional templates for the Discord messages sent after sync. Available variables: `{count}`, `{channelName}`, `{channelId}`

By default, clippings from every synced channel are saved together under `DiscordClippings/`, with no per-channel subfolder.

Channel names cannot contain `\ / : * ? " < > | # ^ [ ]`. The names `.` and `..` are also not allowed. Invalid names are not saved.

## URL Clipping

Every synced message is scanned for a URL (anything starting with `http://` or `https://`). If one is found, the linked page is fetched and saved as an individual Markdown file: `DiscordClippings/YYYYMMDD_HHMMSS_<article title>.md`, using the computer's local time zone when synchronization starts. The title comes from the page's `<title>`, meta/OGP/Twitter title tags, or its first heading; if none of those are found, the Discord message ID is used instead so the file still gets a valid name. If a message has no URL, it is skipped entirely — nothing is written and no notice is shown.

### Discord API behavior

The plugin uses Discord API v10 and requests up to 100 messages at a time. The first synchronization imports the latest 100 messages. Later synchronizations request only the pages needed to reach the saved per-channel cursor. Requests are paginated when more than 100 new messages exist and observe Discord rate-limit responses.

## Development

- [Release procedure](docs/releasing.md)
- [Rust core and TypeScript adapters](docs/rust-core.md)

## References

This plugin was inspired by the following project(s):

- [line_to_obsidian](https://github.com/onikun94/line_to_obsidian)
