import type { StateCreator } from "zustand";

export type LayoutSlice = {
	layout: {
		leftCompressed: boolean;
		centerSplit: number; // 0..1 top ratio
		rightSplit: number; // 0..1 top ratio
		valuesSplit: number; // 0..1 rules/vars ratio
	};
	setLeftCompressed(v: boolean): void;
	setCenterSplit(r: number): void;
	setRightSplit(r: number): void;
	setValuesSplit(r: number): void;
};

export const createLayoutSlice: StateCreator<
	LayoutSlice,
	[],
	[],
	LayoutSlice
> = (set) => ({
	layout: {
		leftCompressed: true,
		centerSplit: 0.6,
		rightSplit: 0.5,
		valuesSplit: 0.5,
	},
	setLeftCompressed: (v) =>
		set((s) => ({ layout: { ...s.layout, leftCompressed: v } })),
	setCenterSplit: (r) =>
		set((s) => ({ layout: { ...s.layout, centerSplit: r } })),
	setRightSplit: (r) =>
		set((s) => ({ layout: { ...s.layout, rightSplit: r } })),
	setValuesSplit: (r) =>
		set((s) => ({ layout: { ...s.layout, valuesSplit: r } })),
});
