export type Rule = {
	id: string;
	namespace: string;
	definition: string;
	enabled: boolean;
};
export type Variable = { id: string; name: string; enabled: boolean };
export type ReplEntry = {
	kind: "history" | "output" | "notice" | "ack";
	text: string;
	id?: string;
	userId?: string;
	ts?: number;
};
export type SidebarNode = {
	id: string;
	name: string;
	isDir?: boolean;
	status?: "connected" | "disconnected";
	kind: "control-plane" | "node-pool" | "node";
	children?: SidebarNode[];
};
export type UserSession = {
	userId: string | null;
	sessionId: string | null;
};
