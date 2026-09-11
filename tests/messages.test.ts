import { describe, expect, test } from "bun:test";
import {
  processed_message as createProcessedMessage,
  type DiscordMessage,
  message_url as messageUrl,
} from "../pkg/parse_message.js";

describe("messageUrl", () => {
  test("finds a URL anywhere in the message", () => {
    expect(messageUrl("https://example.com")).toBe("https://example.com");
    expect(messageUrl("check this out https://example.com/path?q=1 thanks")).toBe(
      "https://example.com/path?q=1",
    );
  });

  test("returns undefined when the message has no URL", () => {
    expect(messageUrl("hello world")).toBeUndefined();
    expect(messageUrl("")).toBeUndefined();
  });
});

describe("createProcessedMessage", () => {
  test("maps a message to the TypeScript domain model", () => {
    const message: DiscordMessage = {
      id: "123",
      content: "content",
      timestamp: "2026-06-21T03:00:00.000Z",
      author: {
        id: "author-id",
        username: "username",
        global_name: "Global name",
      },
      member: { nick: "Nickname" },
    };

    expect(
      createProcessedMessage("# title", "Article Title", message, "Asia/Tokyo"),
    ).toEqual({
      messageId: "123",
      timestamp: "2026-06-21T03:00:00.000Z",
      authorId: "author-id",
      authorName: "Nickname",
      markdown: "# title",
      fileName: "20260621_120000_Article Title",
    });
  });

  test("falls back to the message id when there is no usable title", () => {
    const message: DiscordMessage = {
      id: "123",
      content: "content",
      timestamp: "2026-06-21T03:00:00.000Z",
    };

    expect(
      createProcessedMessage("# title", undefined, message, "Asia/Tokyo")
        .fileName,
    ).toBe("20260621_120000_123");
    expect(
      createProcessedMessage("# title", "   ", message, "Asia/Tokyo").fileName,
    ).toBe("20260621_120000_123");
  });

  test("sanitizes forbidden characters and truncates long titles", () => {
    const message: DiscordMessage = {
      id: "123",
      content: "content",
      timestamp: "2026-06-21T03:00:00.000Z",
    };

    expect(
      createProcessedMessage(
        "# title",
        "A: Title / With * Illegal? Chars",
        message,
        "Asia/Tokyo",
      ).fileName,
    ).toBe("20260621_120000_A- Title - With - Illegal- Chars");
    expect(
      createProcessedMessage("# title", "a".repeat(105), message, "Asia/Tokyo")
        .fileName,
    ).toBe(`20260621_120000_${"a".repeat(100)}`);
  });

  test("falls back through the Discord author fields", () => {
    const base = {
      id: "123",
      content: "content",
      timestamp: "2026-06-21T03:00:00.000Z",
    };

    expect(
      createProcessedMessage(
        "message",
        undefined,
        {
          ...base,
          author: {
            id: "author-id",
            username: "username",
            global_name: "Global name",
          },
        },
        "UTC",
      ).authorName,
    ).toBe("Global name");
    expect(
      createProcessedMessage(
        "message",
        undefined,
        {
          ...base,
          author: { id: "author-id", username: "username" },
        },
        "UTC",
      ).authorName,
    ).toBe("username");
    expect(
      createProcessedMessage(
        "message",
        undefined,
        {
          ...base,
          author: { id: "author-id" },
        },
        "UTC",
      ).authorName,
    ).toBe("author-id");
  });

  test("uses the requested local time zone for file names", () => {
    const processed = createProcessedMessage(
      "message",
      undefined,
      {
        id: "123",
        content: "content",
        timestamp: "2026-06-30T15:30:45.000Z",
      },
      "America/New_York",
    );

    expect(processed.fileName).toBe("20260630_113045_123");
  });

  test("rejects invalid timestamps", () => {
    expect(() =>
      createProcessedMessage(
        "message",
        undefined,
        {
          id: "123",
          content: "content",
          timestamp: "invalid",
        },
        "UTC",
      ),
    ).toThrow('Invalid Discord message timestamp: "invalid".');
  });
});
