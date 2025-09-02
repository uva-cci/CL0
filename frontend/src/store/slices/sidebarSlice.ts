import type { StateCreator } from "zustand";
import type { SidebarNode } from "../types";

export type SidebarSlice = {
	sidebar: { tree: SidebarNode[]; selectedId?: string };
	setSidebarTree(tree: SidebarNode[]): void;
	setSidebarSelected(id?: string): void;
	getSidebarNodeById(id: string): SidebarNode | undefined;
};

export const createSidebarSlice: StateCreator<
	SidebarSlice,
	[],
	[],
	SidebarSlice
> = (set, get) => ({
	sidebar: {
		tree: [],
	},
	setSidebarTree: (tree) => set((s) => ({ sidebar: { ...s.sidebar, tree } })),
	setSidebarSelected: (selectedId) =>
		set((s) => ({ sidebar: { ...s.sidebar, selectedId } })),
	getSidebarNodeById: (id) => {
		const walk = (nodes: SidebarNode[]): SidebarNode | undefined => {
			for (const n of nodes) {
				if (n.id === id) return n;
				if (n.children) {
					const found = walk(n.children);
					if (found) return found;
				}
			}
		};
		return walk(get().sidebar.tree);
	},
});
