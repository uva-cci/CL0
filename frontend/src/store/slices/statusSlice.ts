import type { StateCreator } from "zustand";
import type { RuleStatus, VarStatus } from "../../api/gen/web_pb";

export type StatusSlice = {
	status: {
		rules: (RuleStatus & { id: string })[];
		vars: (VarStatus & { id: string })[];
	};
	setStatus(rules: RuleStatus[], vars: VarStatus[]): void;
	clearStatus(): void;
};

export const createStatusSlice: StateCreator<
	StatusSlice,
	[],
	[],
	StatusSlice
> = (set) => ({
	status: { rules: [], vars: [] },
	setStatus: (rules, vars) =>
		set(() => ({
			status: {
				rules: rules.map((r, i) => ({
					...r,
					id: `${r.namespace}:${r.name}:${i}`,
				})),
				vars: vars.map((v, i) => ({ ...v, id: `${v.name}:${i}` })),
			},
		})),
	clearStatus: () => set(() => ({ status: { rules: [], vars: [] } })),
});
