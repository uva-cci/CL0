import { useEffect, useCallback } from "react";
import { replClient } from "../api/live/transport";
import { useStore } from "../store";
import { Output, Scope, Scope_Kind, Join, Input } from "../api/gen/web_pb";
import { SidebarNode } from "@/store/types";

export const useRepl = () => {
	const pushEntry = useStore((s) => s.pushEntry);
	const clearHistory = useStore((s) => s.clearHistory);
	const session = useStore((s) => s.session);

	const userId = session.userId;
	const selectedScope = useStore((s) => s.sidebar.selectedId);

	const selectedId = useStore((s) => s.sidebar.selectedId);
	const getNodeById = useStore((s) => s.getSidebarNodeById);

	const selectedNode = selectedId ? getNodeById(selectedId) : undefined;

	const kindFromSidebar = (node?: SidebarNode): Scope_Kind => {
		if (!node) return Scope_Kind.KIND_UNSPECIFIED;
		switch (node.kind) {
			case "control-plane":
				return Scope_Kind.CONTROL_PLANE;
			case "node-pool":
				return Scope_Kind.NODE_POOL;
			case "node":
				return Scope_Kind.NODE;
			default:
				return Scope_Kind.KIND_UNSPECIFIED;
		}
	};

	useEffect(() => {
		if (!userId || !selectedScope) return;
		let aborted = false;

		(async () => {
			const join: Join = {
				userId,
				scope: {
					kind: kindFromSidebar(selectedNode),
					id: selectedNode?.id ?? "",
					$typeName: "web.Scope",
				},
				sinceId: "",
				$typeName: "web.Join",
			};

			for await (const evt of replClient.subscribe(join)) {
				if (aborted) break;
				if (!evt.kind) continue;

				switch (evt.kind.case) {
					case "history":
						evt.kind.value.items.forEach((o: Output) => {
							pushEntry({
								kind: "history",
								text: o.stdout,
								id: o.id,
								userId: o.userId,
								ts: Number(o.unixTs),
							});
						});
						break;

					case "output":
						pushEntry({
							kind: "output",
							text: evt.kind.value.stdout,
							id: evt.kind.value.id,
							userId: evt.kind.value.userId,
							ts: Number(evt.kind.value.unixTs),
						});
						break;

					case "notice":
						pushEntry({
							kind: "notice",
							text: `* ${evt.kind.value.text}`,
						});
						break;

					case "ack":
						pushEntry({
							kind: "ack",
							text: `✔ ack ${evt.kind.value.inputEcho}`,
							id: evt.kind.value.outputId,
						});
						break;
				}
			}
		})();

		return () => {
			aborted = true;
		};
	}, [userId, selectedScope, pushEntry]);

	const send = useCallback(
		async (code: string) => {
			if (!userId || !selectedScope) return;

			const req: Input = {
				userId,
				scope: {
					kind: kindFromSidebar(selectedNode),
					id: selectedNode?.id ?? "",
					$typeName: "web.Scope",
				},
				code,
				$typeName: "web.Input",
			};
			await replClient.sendCommand(req);
		},
		[userId, selectedScope],
	);

	return { send, clearHistory };
};
