//
//  fetch.ts
//  Project: Floe
//
//  Copyright (Floe) © 2026 René Jiménez
//  Licensed under the GNU AGPLv3

// Node's fetch rejects with a TypeError whose cause carries the socket's code; Bun puts the code on the error itself.
// Extensions are written against Node, and read the code there to say that the server they need is not running.
export function withNodeCause(fetch: typeof globalThis.fetch): typeof globalThis.fetch {
  const wrapped = (...args: Parameters<typeof globalThis.fetch>) =>
    fetch(...args).catch((error: unknown) => {
      if (!(error instanceof TypeError)) throw error;
      const own = (error as { code?: unknown }).code;
      // Bun 1.4.2 names a refused connection in its own word, where later versions use Node's.
      const code = own === "ConnectionRefused" ? "ECONNREFUSED" : own;
      const cause = error.cause;
      if (typeof code === "string" && !(typeof cause === "object" && cause !== null && "code" in cause)) {
        Object.assign(error, { code });
        error.cause = Object.assign(new Error(error.message), { code });
      }
      throw error;
    });
  // preconnect, and whatever else Bun hangs on fetch.
  return Object.assign(wrapped, fetch);
}
