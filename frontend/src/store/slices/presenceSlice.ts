import type { StateCreator } from "zustand";
import type { Presence } from "../../api/gen/web_pb";

export type PresenceSlice = {
	presence: { users: Presence[] };
	setPresence(users: Presence[]): void;
	upsertPresence(user: Presence): void;
	removePresence(userId: string): void;
};

export const createPresenceSlice: StateCreator<
	PresenceSlice,
	[],
	[],
	PresenceSlice
> = (set) => ({
	presence: { users: [] },
	setPresence: (users) => set(() => ({ presence: { users } })),
	upsertPresence: (user) =>
		set((s) => {
			const existing = s.presence.users.filter(
				(u) => u.userId !== user.userId,
			);
			return { presence: { users: [...existing, user] } };
		}),
	removePresence: (userId) =>
		set((s) => ({
			presence: {
				users: s.presence.users.filter((u) => u.userId !== userId),
			},
		})),
});
