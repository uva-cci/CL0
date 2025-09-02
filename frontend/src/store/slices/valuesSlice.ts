import type { StateCreator } from "zustand";
import type { Rule, Variable } from "../types";

export type ValuesSlice = {
	values: { rules: Rule[]; variables: Variable[] };
	setRules(rules: Rule[]): void;
	setVariables(vars: Variable[]): void;
};

export const createValuesSlice: StateCreator<
	ValuesSlice,
	[],
	[],
	ValuesSlice
> = (set) => ({
	values: { rules: [], variables: [] },
	setRules: (rules) => set((s) => ({ values: { ...s.values, rules } })),
	setVariables: (variables) =>
		set((s) => ({ values: { ...s.values, variables } })),
});
