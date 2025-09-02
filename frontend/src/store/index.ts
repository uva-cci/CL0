import { create } from "zustand";
import { devtools, persist, createJSONStorage } from "zustand/middleware";
import type { LayoutSlice } from "./slices/layoutSlice";
import type { SidebarSlice } from "./slices/sidebarSlice";
import type { ReplSlice } from "./slices/replSlice";
import type { PolicySlice } from "./slices/policySlice";
import type { ValuesSlice } from "./slices/valuesSlice";
import { createLayoutSlice } from "./slices/layoutSlice";
import { createSidebarSlice } from "./slices/sidebarSlice";
import { createReplSlice } from "./slices/replSlice";
import { createPolicySlice } from "./slices/policySlice";
import { createValuesSlice } from "./slices/valuesSlice";
import { createSessionSlice, SessionSlice } from "./slices/sessionSlice";
import { createPresenceSlice, PresenceSlice } from "./slices/presenceSlice";
import { createStatusSlice, StatusSlice } from "./slices/statusSlice";

export type RootState = LayoutSlice &
	SidebarSlice &
	ReplSlice &
	PolicySlice &
	ValuesSlice &
	SessionSlice &
	PresenceSlice &
	StatusSlice;

export const useStore = create<RootState>()(
	devtools(
		// persist(
			(...a) => ({
				...createLayoutSlice(...a),
				...createSidebarSlice(...a),
				...createReplSlice(...a),
				...createPolicySlice(...a),
				...createValuesSlice(...a),
                ...createSessionSlice(...a),
                ...createPresenceSlice(...a),
                ...createStatusSlice(...a)
			}),
			// {
			// 	name: "app-cache",
			// 	storage: createJSONStorage(() => localStorage),
			// 	partialize: (s) => ({
			// 		layout: s.layout,
			// 		sidebar: s.sidebar,
			// 		policy: s.policy,
			// 	}),
			// },
		// ),
	),
);
