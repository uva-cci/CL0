import type { StateCreator } from "zustand";

export type PolicySlice = {
	policy: { json: string };
	setPolicy(json: string): void;
};

export const createPolicySlice: StateCreator<
	PolicySlice,
	[],
	[],
	PolicySlice
> = (set) => ({
	policy: { json: '{\n  "example": true\n}' },
	setPolicy: (json) => set(() => ({ policy: { json } })),
});
