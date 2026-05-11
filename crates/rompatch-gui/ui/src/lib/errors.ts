// IPC errors arrive as `{ kind, message }` objects (see GuiError's
// Serialize impl in src/error.rs). When `invoke` rejects we get the
// deserialized payload directly. Anything else (network, unexpected
// runtime throw) falls back to String coercion.

export function formatIpcError(err: unknown): string {
  if (err && typeof err === 'object' && 'message' in err) {
    const msg = (err as { message: unknown }).message;
    if (typeof msg === 'string') return msg;
  }
  return String(err);
}
