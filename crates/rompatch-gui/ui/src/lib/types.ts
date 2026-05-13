// Mirrors of the JSON shapes our Rust IPC commands return.
//
// Serde defaults: unit enum variants serialize as their PascalCase name
// (e.g. `FormatKind::Ips` -> `"Ips"`). Struct fields serialize in
// snake_case. Keep these aligned with rompatch-core's derives.

export type FormatKind =
  | 'Ips'
  | 'Ups'
  | 'Bps'
  | 'Pmsr'
  | 'ApsGba'
  | 'ApsN64'
  | 'Ppf'
  | 'Rup'
  | 'Bdf';

export type HeaderKind = 'SmcSnes' | 'INes' | 'Fds' | 'Lynx';

export type HashAlgo = 'Crc32' | 'Md5' | 'Sha1' | 'Adler32';

export type HashCheckKind = 'Input' | 'Output';

export type ChecksumFamily = 'GameBoy' | 'MegaDrive';

export interface HashSpec {
  algo: HashAlgo;
  expected_hex: string;
}

export interface ApplyOptions {
  strip_header: boolean;
  fix_checksum: boolean;
  verify_input: HashSpec | null;
  verify_output: HashSpec | null;
  format_override: FormatKind | null;
}

export interface PatchInfo {
  format: FormatKind;
  patch_size: number;
  fields: Array<[string, string]>;
}

export interface HashReport {
  crc32: string;
  md5: string;
  sha1: string;
  adler32: string;
  file_size: number;
}

export interface ApplyReport {
  format: FormatKind;
  out_path: string;
  out_size: number;
  stripped_header: HeaderKind | null;
  fixed_checksum: ChecksumFamily | null;
}

export interface IpcError {
  kind: 'io' | 'apply' | 'patch' | 'json' | 'library' | 'tauri';
  message: string;
}

// ---------- library ----------

export interface LibraryRomEntry {
  id: string;
  rom_hash: string;
  rom_name: string;
  rom_size: number;
  header: HeaderKind | null;
  added_at: string;
}

export interface LibraryEntry {
  id: string;
  source_rom_hash: string;
  source_rom_name: string;
  source_rom_size: number;
  patch_hash: string;
  patch_name: string;
  patch_format: FormatKind;
  output_hash: string;
  output_name: string;
  output_size: number;
  header: HeaderKind | null;
  fixed_checksum: ChecksumFamily | null;
  applied_at: string; // ISO 8601 UTC
  apply_options: ApplyOptions;
}

export type VerifyStatus = 'match' | 'mismatch' | 'missing';

export type RevealTarget = 'source' | 'patch' | 'output';

export interface LibraryRecordArgs {
  source_path: string;
  patch_path: string;
  output_path: string;
  format: FormatKind;
  header: HeaderKind | null;
  fixed_checksum: ChecksumFamily | null;
  apply_options: ApplyOptions;
}

// Display names for UI labels.

export const FORMAT_DISPLAY: Record<FormatKind, string> = {
  Ips: 'IPS',
  Ups: 'UPS',
  Bps: 'BPS',
  Pmsr: 'PMSR',
  ApsGba: 'APS-GBA',
  ApsN64: 'APS-N64',
  Ppf: 'PPF',
  Rup: 'RUP',
  Bdf: 'BDF',
};

export const HEADER_DISPLAY: Record<HeaderKind, string> = {
  SmcSnes: 'SMC (SNES)',
  INes: 'iNES (NES)',
  Fds: 'FDS',
  Lynx: 'LYNX',
};

export const HASH_ALGO_DISPLAY: Record<HashAlgo, string> = {
  Crc32: 'CRC32',
  Md5: 'MD5',
  Sha1: 'SHA-1',
  Adler32: 'Adler-32',
};

export const CHECKSUM_FAMILY_DISPLAY: Record<ChecksumFamily, string> = {
  GameBoy: 'Game Boy',
  MegaDrive: 'Mega Drive',
};
