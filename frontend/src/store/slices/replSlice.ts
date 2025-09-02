import type { StateCreator } from "zustand";
import type { ReplEntry } from "../types";

export type ReplSlice = {
	repl: { history: ReplEntry[]; isStreaming: boolean };
	pushEntry(e: ReplEntry): void;
	setStreaming(v: boolean): void;
	clearHistory(): void;
};

export const createReplSlice: StateCreator<ReplSlice, [], [], ReplSlice> = (
	set,
) => ({
	repl: { history: [], isStreaming: false },
	pushEntry: (e) =>
		set((s) => ({ repl: { ...s.repl, history: [...s.repl.history, e] } })),
	setStreaming: (v) => set((s) => ({ repl: { ...s.repl, isStreaming: v } })),
	clearHistory: () => set((s) => ({ repl: { ...s.repl, history: [] } })),
});
