import { Notice, requestUrl } from "obsidian";
import {
  convert_html as convertHtml,
  extract_article_title as extractArticleTitle,
  processed_message as createProcessedMessage,
  type DiscordMessage,
  type InitOutput,
  type ProcessedMessage,
  message_url as messageUrl,
} from "../pkg/parse_message.js";
import { initWasmCore } from "./wasmCore";

export async function initWasmBridge(): Promise<InitOutput> {
  try {
    return await initWasmCore();
  } catch (error) {
    new Notice("WASM initialization failed.");
    throw error;
  }
}

export async function parseMessageWasm(
  message: DiscordMessage,
  timeZone: string,
): Promise<ProcessedMessage | undefined> {
  await initWasmBridge();

  const url = messageUrl(message.content);
  if (!url) {
    return undefined;
  }

  const html = await fetchUrlContent(url);
  let markdown: string;
  let title: string | undefined;
  try {
    markdown = convertHtml(url, html);
    title = extractArticleTitle(url, html);
  } catch (error) {
    throw new Error("Failed to convert URL content to Markdown.", {
      cause: error,
    });
  }

  return createProcessedMessage(markdown, title, message, timeZone);
}

async function fetchUrlContent(value: string): Promise<string> {
  let url: URL;
  try {
    url = new URL(value);
  } catch (error) {
    throw new Error("URL command requires a valid absolute URL.", {
      cause: error,
    });
  }

  if (url.protocol !== "https:") {
    throw new Error("Only HTTPS URLs are supported.");
  }

  try {
    const response = await requestUrl({
      url: url.toString(),
      method: "GET",
      headers: { "User-Agent": "Obsidian Discord Sender" },
    });
    return response.text;
  } catch (error) {
    throw new Error("Failed to fetch URL content.", { cause: error });
  }
}
