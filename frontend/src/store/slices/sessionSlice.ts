import type { StateCreator } from "zustand";

export type SessionSlice = {
    session: { userId: string | null; sessionId: string | null };
    setSession: (userId: string, sessionId: string) => void;
    clearSession: () => void;
};

export const createSessionSlice: StateCreator<SessionSlice> = (set) => ({
    session: { userId: null, sessionId: null },
    setSession: (userId, sessionId) => set({ session: { userId, sessionId } }),
    clearSession: () => set({ session: { userId: null, sessionId: null } }),
});
