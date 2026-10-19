import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { Database } from "bun:sqlite";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createLocalStorage as openLocalStorage, closeLocalStorage, storagePersistence } from "../local-storage";
const storages = new Set<ReturnType<typeof openLocalStorage>>();
function createLocalStorage(file: () => string) {
    const storage = openLocalStorage(file);
    storages.add(storage);
    return storage;
}

let folder: string;
let file: string;
const original = { ...storagePersistence };

beforeEach(() => {
    folder = fs.mkdtempSync(path.join(os.tmpdir(), "floe-local-storage-"));
    file = path.join(folder, "local-storage.sqlite");
});

afterEach(async () => {
    await Promise.all([...storages].map(closeLocalStorage));
    storages.clear();
    Object.assign(storagePersistence, original);
    fs.rmSync(folder, { recursive: true, force: true });
});

/// What is stored, read by a connection of its own the way another process would read it.
const onDisk = () => {
    const database = new Database(file, { readonly: true });
    try {
        return Object.fromEntries(
            (database.prepare("select key, value from items").all() as { key: string; value: string }[]).map(({ key, value }) => [key, JSON.parse(value)]),
        );
    } finally {
        database.close(true);
    }
};

describe("LocalStorage", () => {
    test("values come back with their type, and removing and clearing work", async () => {
        const storage = createLocalStorage(() => file);
        await storage.setItem("name", "floe");
        await storage.setItem("count", 3);
        await storage.setItem("on", true);
        expect(await storage.getItem("name")).toBe("floe");
        expect(await storage.getItem<number>("count")).toBe(3);
        expect(await storage.getItem<boolean>("on")).toBe(true);
        expect(await storage.getItem("missing")).toBeUndefined();
        expect(await storage.allItems()).toEqual({ name: "floe", count: 3, on: true });
        await storage.removeItem("count");
        expect(onDisk()).toEqual({ name: "floe", on: true });
        await storage.clear();
        expect(onDisk()).toEqual({});
    });

    test("a file from before this change still loads, and is gone once it does", async () => {
        fs.writeFileSync(path.join(folder, "local-storage.json"), JSON.stringify({ token: "abc", seen: 2 }));
        const storage = createLocalStorage(() => file);
        expect(await storage.getItem("token")).toBe("abc");
        expect(await storage.allItems()).toEqual({ token: "abc", seen: 2 });
        expect(onDisk()).toEqual({ token: "abc", seen: 2 });
        expect(fs.existsSync(path.join(folder, "local-storage.json"))).toBe(false);
    });

    test("a write that fails leaves the previous values whole", async () => {
        let database: Database | undefined;
        storagePersistence.open = (target) => (database = new Database(target));
        const storage = createLocalStorage(() => file);
        await storage.setItem("kept", "yes");
        // The connection dies underneath, the way a killed process's would have: the write fails and nothing of it is kept.
        database!.close(true);
        await expect(storage.setItem("lost", "x".repeat(4096))).rejects.toThrow();
        expect(await createLocalStorage(() => file).getItem("kept")).toBe("yes");
        expect(await createLocalStorage(() => file).getItem("lost")).toBeUndefined();
    });

    test("a file that cannot be read is moved aside, not overwritten", async () => {
        fs.writeFileSync(path.join(folder, "local-storage.json"), '{"token": "abc", "half');
        const storage = createLocalStorage(() => file);
        expect(await storage.allItems()).toEqual({});
        await storage.setItem("fresh", 1);
        expect(onDisk()).toEqual({ fresh: 1 });
        const aside = fs.readdirSync(folder).filter((name) => name.includes(".unreadable-"));
        expect(aside).toHaveLength(1);
        expect(fs.readFileSync(path.join(folder, aside[0]), "utf8")).toBe('{"token": "abc", "half');
    });

    test("a database another process holds is waited out, never moved aside", async () => {
        const storage = createLocalStorage(() => file);
        await storage.setItem("kept", "yes");
        const other = new Database(file);
        other.exec("BEGIN EXCLUSIVE");
        // While the other process holds the database, the write waits for it: it neither fails nor
        // quarantines the file, and the waiting leaves the event loop free to run.
        let done = false;
        const during = storage.setItem("during", 1).then(() => {
            done = true;
        });
        await Bun.sleep(60);
        expect(done).toBe(false);
        expect(fs.readdirSync(folder).filter((name) => name.includes(".unreadable-"))).toHaveLength(0);
        other.exec("COMMIT");
        await during;
        expect(done).toBe(true);
        expect(await storage.getItem<number>("during")).toBe(1);
        expect(await storage.getItem("kept")).toBe("yes");
        other.close(true);
    });

    test("an import never covers a value written after the legacy file was read", async () => {
        // The state a race between two processes leaves behind: the newer value is already in the
        // table when the stale import finally runs, and the newer of the two is the one that stays.
        fs.writeFileSync(path.join(folder, "local-storage.json"), JSON.stringify({ key: "legacy" }));
        const storage = createLocalStorage(() => file);
        const raw = new Database(file);
        raw.exec("CREATE TABLE IF NOT EXISTS items (key TEXT PRIMARY KEY, value TEXT NOT NULL)");
        raw.exec("CREATE TABLE IF NOT EXISTS migration (done INTEGER NOT NULL)");
        raw.exec(`INSERT INTO items (key, value) VALUES ('key', '"newer"')`);
        raw.close(true);
        expect(await storage.getItem("key")).toBe("newer");
        expect(onDisk()).toEqual({ key: "newer" });
        expect(fs.existsSync(path.join(folder, "local-storage.json"))).toBe(false);
    });

    test("a migration that has run is never run again, so a deleted key stays deleted", async () => {
        fs.writeFileSync(path.join(folder, "local-storage.json"), JSON.stringify({ key: "legacy" }));
        const storage = createLocalStorage(() => file);
        expect(await storage.getItem("key")).toBe("legacy");
        await storage.removeItem("key");
        // The legacy file is back, as one that survived the first import or was written again would be:
        // an open against it must not import the deleted key back in.
        fs.writeFileSync(path.join(folder, "local-storage.json"), JSON.stringify({ key: "legacy" }));
        expect(await createLocalStorage(() => file).getItem("key")).toBeUndefined();
        expect(onDisk()).toEqual({});
        expect(fs.existsSync(path.join(folder, "local-storage.json"))).toBe(false);
        // One import, one claim: the marker holds a single row for INSERT OR IGNORE to have ignored.
        const raw = new Database(file);
        expect((raw.prepare("SELECT COUNT(*) AS count FROM migration").get() as { count: number }).count).toBe(1);
        raw.close(true);
    });

    test("two processes that read the same legacy file import it once", async () => {
        fs.writeFileSync(path.join(folder, "local-storage.json"), JSON.stringify({ key: "legacy" }));
        const view = createLocalStorage(() => file);
        const background = createLocalStorage(() => file);
        await Promise.all([view.allItems(), background.allItems()]);
        expect(await view.getItem("key")).toBe("legacy");
        expect(await background.getItem("key")).toBe("legacy");
        expect(onDisk()).toEqual({ key: "legacy" });
        expect(fs.existsSync(path.join(folder, "local-storage.json"))).toBe(false);
    });

    test("a database that cannot be read is moved aside, not overwritten", async () => {
        fs.writeFileSync(file, "not a database at all");
        const storage = createLocalStorage(() => file);
        expect(await storage.allItems()).toEqual({});
        await storage.setItem("fresh", 1);
        expect(await storage.getItem<number>("fresh")).toBe(1);
    });

    test("a database that cannot be written is left where it is", async () => {
        await createLocalStorage(() => file).setItem("kept", "yes");
        const real = storagePersistence.open;
        storagePersistence.open = (target) => new Database(target, { readonly: true });
        try {
            await expect(createLocalStorage(() => file).setItem("lost", 1)).rejects.toThrow();
        } finally {
            storagePersistence.open = real;
        }
        expect(fs.readdirSync(folder).filter((name) => name.includes(".unreadable-"))).toHaveLength(0);
        expect(await createLocalStorage(() => file).getItem("kept")).toBe("yes");
    });

    test("what another process of the extension wrote is seen, and a write keeps it", async () => {
        const view = createLocalStorage(() => file);
        const background = createLocalStorage(() => file);
        await view.setItem("fromView", 1);
        await background.setItem("fromBackground", 2);
        expect(await view.getItem<number>("fromBackground")).toBe(2);
        await view.setItem("later", 3);
        expect(onDisk()).toEqual({ fromView: 1, fromBackground: 2, later: 3 });
    });

    test("processes saving at the same moment lose none of each other's keys", async () => {
        const writer = path.join(import.meta.dir, "storage-writer.ts");
        const writers = ["view", "background", "menu", "extra"].map((prefix) => Bun.spawn(["bun", writer, file, prefix, "40"], { stdout: "ignore", stderr: "inherit" }));
        expect(await Promise.all(writers.map((process) => process.exited))).toEqual([0, 0, 0, 0]);
        expect(Object.keys(onDisk())).toHaveLength(160);
        expect(fs.existsSync(`${file}.lock`)).toBe(false);
    }, 30000);

    test("a lock left over from before this change is of no concern", async () => {
        fs.writeFileSync(`${file}.lock`, "");
        const storage = createLocalStorage(() => file);
        await storage.setItem("afterStale", 2);
        expect(onDisk()).toEqual({ afterStale: 2 });
    });

    test("removing a key that is not there changes nothing", async () => {
        const storage = createLocalStorage(() => file);
        await storage.setItem("a", 1);
        await storage.removeItem("missing");
        expect(await storage.allItems()).toEqual({ a: 1 });
    });

    test("changing what allItems returned does not change what is stored", async () => {
        const storage = createLocalStorage(() => file);
        await storage.setItem("a", 1);
        const items = await storage.allItems<Record<string, number>>();
        items.a = 99;
        expect(await storage.getItem<number>("a")).toBe(1);
    });

    test("no files are left over after a write", async () => {
        const storage = createLocalStorage(() => file);
        await storage.setItem("a", 1);
        expect(fs.readdirSync(folder)).toEqual(["local-storage.sqlite"]);
    });
});
