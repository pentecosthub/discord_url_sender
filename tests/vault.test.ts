import { describe, expect, test } from "bun:test";
import type { TFile, TFolder, Vault } from "obsidian";
import {
  processed_message as createProcessedMessage,
  type ProcessedMessage,
} from "../pkg/parse_message.js";
import { saveProcessedMessages } from "../src/vault";

function createVaultMock() {
  const files = new Map<string, string>();
  const folders = new Set<string>();
  const createdPaths: string[] = [];

  const asFile = (path: string) =>
    ({
      path,
      name: path.slice(path.lastIndexOf("/") + 1),
    }) as TFile;
  const asFolder = (path: string) =>
    ({
      path,
      name: path.slice(path.lastIndexOf("/") + 1),
      children: [
        ...Array.from(files.keys())
          .filter(
            (filePath) => filePath.slice(0, filePath.lastIndexOf("/")) === path,
          )
          .map(asFile),
        ...Array.from(folders)
          .filter(
            (folderPath) =>
              folderPath !== path &&
              folderPath.slice(0, folderPath.lastIndexOf("/")) === path,
          )
          .map(
            (folderPath) =>
              ({
                path: folderPath,
                name: folderPath.slice(folderPath.lastIndexOf("/") + 1),
                children: [],
              }) as unknown as TFolder,
          ),
      ],
    }) as unknown as TFolder;

  const vault = {
    getFolderByPath: (path: string) =>
      folders.has(path) ? asFolder(path) : null,
    getFileByPath: (path: string) => (files.has(path) ? asFile(path) : null),
    getAbstractFileByPath: (path: string) => {
      if (files.has(path)) {
        return asFile(path);
      }
      return folders.has(path) ? asFolder(path) : null;
    },
    createFolder: async (path: string) => {
      folders.add(path);
      return asFolder(path);
    },
    create: async (path: string, content: string) => {
      createdPaths.push(path);
      files.set(path, content);
      return asFile(path);
    },
  } satisfies Pick<
    Vault,
    "getFolderByPath" | "getFileByPath" | "getAbstractFileByPath" | "createFolder" | "create"
  >;

  return { vault: vault as Vault, files, folders, createdPaths };
}

function createMessage(
  id: string,
  timestamp: string,
  markdown = "hello",
  timeZone = "Asia/Tokyo",
): ProcessedMessage {
  return createProcessedMessage(
    markdown,
    {
      id,
      content: markdown,
      timestamp,
      author: { id: `author-${id}`, username: "Alice" },
    },
    timeZone,
  );
}

describe("saveProcessedMessages", () => {
  test("saves a new clipping as an individual file", async () => {
    const { vault, files, folders } = createVaultMock();

    const count = await saveProcessedMessages(
      vault,
      "DiscordClippings/general",
      [createMessage("123", "2026-06-21T03:00:00.000Z", "# Example")],
    );

    expect(count).toBe(1);
    expect(folders).toEqual(
      new Set(["DiscordClippings", "DiscordClippings/general"]),
    );
    expect(files).toEqual(
      new Map([
        ["DiscordClippings/general/20260621_120000_123.md", "# Example"],
      ]),
    );
  });

  test("uses local time for new clipping file names", async () => {
    const { vault, files } = createVaultMock();
    const timeZone = "America/New_York";

    const count = await saveProcessedMessages(vault, "DiscordClippings/general", [
      createMessage("123", "2026-06-30T15:30:45.000Z", "hello", timeZone),
    ]);

    expect(count).toBe(1);
    expect(files.has("DiscordClippings/general/20260630_113045_123.md")).toBe(
      true,
    );
  });

  test("does not duplicate legacy JST clippings after changing time zone", async () => {
    const { vault, files, folders } = createVaultMock();
    folders.add("DiscordClippings");
    folders.add("DiscordClippings/general");
    files.set(
      "DiscordClippings/general/20260701_003045_123.md",
      "Existing clipping",
    );
    const timeZone = "America/New_York";

    const count = await saveProcessedMessages(vault, "DiscordClippings/general", [
      createMessage("123", "2026-06-30T15:30:45.000Z", "Replacement", timeZone),
    ]);

    expect(count).toBe(0);
    expect(files.size).toBe(1);
    expect(
      files.has("DiscordClippings/general/20260630_113045_123.md"),
    ).toBe(false);
  });

  test("does not save the same message id twice", async () => {
    const { vault, files } = createVaultMock();
    const message = createMessage("123", "2026-06-29T12:34:00.000Z");

    const count = await saveProcessedMessages(vault, "DiscordClippings/general", [
      message,
      message,
    ]);

    expect(count).toBe(1);
    expect(files.size).toBe(1);
  });

  test("rejects folder collisions at clipping file paths", async () => {
    const { vault, files, folders, createdPaths } = createVaultMock();
    const path = "DiscordClippings/general/20260629_213400_123.md";
    folders.add("DiscordClippings");
    folders.add("DiscordClippings/general");
    folders.add(path);

    await expect(
      saveProcessedMessages(vault, "DiscordClippings/general", [
        createMessage("123", "2026-06-29T12:34:00.000Z", "content"),
      ]),
    ).rejects.toThrow(
      `a folder exists at "${path}"; move or rename it, then sync again`,
    );
    expect(files.size).toBe(0);
    expect(createdPaths).toEqual([]);
  });

  test("ignores unrelated files and folders when detecting existing ids", async () => {
    const { vault, files, folders } = createVaultMock();
    folders.add("DiscordClippings");
    folders.add("DiscordClippings/general");
    // A folder that happens to look like an individual clipping file must not
    // be mistaken for an existing id, and neither should a mismatched name.
    folders.add("DiscordClippings/general/20260629_213400_999.md");
    files.set("DiscordClippings/general/archive_123.md", "User note");

    const count = await saveProcessedMessages(vault, "DiscordClippings/general", [
      createMessage("123", "2026-06-29T12:34:00.000Z"),
    ]);

    expect(count).toBe(1);
    expect(files.has("DiscordClippings/general/20260629_213400_123.md")).toBe(
      true,
    );
  });

  test("keeps two channel clippings separate", async () => {
    const { vault, files } = createVaultMock();

    await saveProcessedMessages(vault, "DiscordClippings/first", [
      createMessage("123", "2026-06-29T12:34:00.000Z"),
    ]);
    await saveProcessedMessages(vault, "DiscordClippings/second", [
      createMessage("124", "2026-06-29T12:35:00.000Z"),
    ]);

    expect(files.has("DiscordClippings/first/20260629_213400_123.md")).toBe(
      true,
    );
    expect(files.has("DiscordClippings/second/20260629_213500_124.md")).toBe(
      true,
    );
  });

  test("rejects invalid timestamps before writing files", () => {
    expect(() =>
      createProcessedMessage(
        "hello",
        {
          id: "123",
          content: "hello",
          timestamp: "invalid",
        },
        "UTC",
      ),
    ).toThrow('Invalid Discord message timestamp: "invalid".');
  });
});
