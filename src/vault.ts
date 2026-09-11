import type { Vault } from "obsidian";
import {
  plan_message_storage,
  type ProcessedMessage,
  type StorageInput,
} from "../pkg/parse_message.js";

export class MessageStorageError extends Error {
  override name = "MessageStorageError";
}

export async function saveProcessedMessages(
  vault: Vault,
  clippingDirectory: string,
  messages: readonly ProcessedMessage[],
): Promise<number> {
  const input: StorageInput = {
    clippingDirectory,
    messages: [...messages],
  };
  const plan = plan_message_storage(input);

  let savedCount = 0;
  for (const write of plan.writes) {
    await ensureDir(vault, clippingDirectory);
    const existing = vault.getAbstractFileByPath(write.path);
    if (existing) {
      if (!vault.getFileByPath(write.path)) {
        throw new MessageStorageError(
          `a folder exists at "${write.path}"; move or rename it, then sync again`,
        );
      }
      continue;
    }
    await vault.create(write.path, write.content);
    savedCount++;
  }
  return savedCount;
}

async function ensureDir(vault: Vault, path: string): Promise<void> {
  if (vault.getFolderByPath(path)) {
    return;
  }

  if (vault.getAbstractFileByPath(path)) {
    throw new MessageStorageError(
      `a file blocks the directory "${path}"; move or rename it, then sync again`,
    );
  }

  const parent = path.split("/").slice(0, -1).join("/");
  if (parent) {
    await ensureDir(vault, parent);
  }
  await vault.createFolder(path);
}
