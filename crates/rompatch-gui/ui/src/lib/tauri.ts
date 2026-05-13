// Typed wrappers around `invoke()` for every Rust command. Centralizing
// here means components don't repeat command names or argument shapes,
// and renames stay caught at compile time.

import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import type {
  ApplyOptions,
  ApplyReport,
  FormatKind,
  HashReport,
  HeaderKind,
  LibraryEntry,
  LibraryRecordArgs,
  LibraryRomEntry,
  PatchInfo,
  RevealTarget,
  VerifyStatus,
} from './types';

export async function detectPatchFormat(patchPath: string): Promise<FormatKind | null> {
  return invoke<FormatKind | null>('detect_patch_format', { patchPath });
}

export async function describePatch(patchPath: string): Promise<PatchInfo> {
  return invoke<PatchInfo>('describe_patch', { patchPath });
}

export async function detectRomHeader(romPath: string): Promise<HeaderKind | null> {
  return invoke<HeaderKind | null>('detect_rom_header', { romPath });
}

export async function computeHashes(filePath: string): Promise<HashReport> {
  return invoke<HashReport>('compute_hashes', { filePath });
}

export async function applyPatch(
  romPath: string,
  patchPath: string,
  outPath: string,
  options: ApplyOptions,
): Promise<ApplyReport> {
  return invoke<ApplyReport>('apply_patch', {
    romPath,
    patchPath,
    outPath,
    options,
  });
}

export async function defaultOutputPath(romPath: string): Promise<string> {
  return invoke<string>('default_output_path', { romPath });
}

// File picker helpers. Tauri returns string|null for single-pick.

export async function pickFile(title?: string): Promise<string | null> {
  const result = await open({
    multiple: false,
    directory: false,
    title: title ?? 'Select file',
  });
  return typeof result === 'string' ? result : null;
}

export async function pickSavePath(
  defaultPath?: string,
  title?: string,
): Promise<string | null> {
  const result = await save({
    title: title ?? 'Save output',
    defaultPath,
  });
  return result ?? null;
}

export async function pickDirectory(
  defaultPath?: string,
  title?: string,
): Promise<string | null> {
  const result = await open({
    directory: true,
    multiple: false,
    defaultPath,
    title: title ?? 'Select folder',
  });
  return typeof result === 'string' ? result : null;
}

// ---------- library ----------

export async function libraryRoot(): Promise<string> {
  return invoke<string>('library_root');
}

export async function librarySetRoot(newRoot: string): Promise<string> {
  return invoke<string>('library_set_root', { newRoot });
}

export async function libraryList(): Promise<LibraryEntry[]> {
  return invoke<LibraryEntry[]>('library_list');
}

export async function libraryListRoms(): Promise<LibraryRomEntry[]> {
  return invoke<LibraryRomEntry[]>('library_list_roms');
}

export async function libraryImportRom(romPath: string): Promise<LibraryRomEntry> {
  return invoke<LibraryRomEntry>('library_import_rom', { romPath });
}

export async function libraryRomPath(romHash: string): Promise<string> {
  return invoke<string>('library_rom_path', { romHash });
}

export async function libraryRecord(args: LibraryRecordArgs): Promise<LibraryEntry> {
  return invoke<LibraryEntry>('library_record', { args });
}

export async function libraryVerify(entryId: string): Promise<VerifyStatus> {
  return invoke<VerifyStatus>('library_verify', { entryId });
}

export async function libraryReapply(entryId: string): Promise<VerifyStatus> {
  return invoke<VerifyStatus>('library_reapply', { entryId });
}

export async function libraryReveal(
  entryId: string,
  target: RevealTarget,
): Promise<void> {
  return invoke<void>('library_reveal', { entryId, target });
}

export async function libraryDeleteEntry(entryId: string): Promise<void> {
  return invoke<void>('library_delete_entry', { entryId });
}

export async function libraryDeleteRom(romHash: string): Promise<void> {
  return invoke<void>('library_delete_rom', { romHash });
}

export async function libraryExport(entryId: string, destPath: string): Promise<void> {
  return invoke<void>('library_export', { entryId, destPath });
}

export async function libraryLookupByPatchHash(
  patchPath: string,
): Promise<LibraryEntry[]> {
  return invoke<LibraryEntry[]>('library_lookup_by_patch_hash', { patchPath });
}
