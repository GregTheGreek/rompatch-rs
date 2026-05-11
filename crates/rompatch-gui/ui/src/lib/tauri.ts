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
  PatchInfo,
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
