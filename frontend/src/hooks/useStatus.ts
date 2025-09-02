import { useEffect } from "react";
import { statusClient } from "../api/live/transport";
import { useStore } from "../store";
import { Scope, Scope_Kind } from "../api/gen/web_pb";

export const useStatus = () => {
	const selectedId = useStore((s) => s.sidebar.selectedId);
	const setStatus = useStore((s) => s.setStatus);
	const clearStatus = useStore((s) => s.clearStatus);

	useEffect(() => {
		let aborted = false;

		const fetchStatus = async () => {
			if (!selectedId) {
				clearStatus();
				return;
			}

			const scope: Scope = {
				kind: Scope_Kind.NODE,
				id: selectedId,
				$typeName: "web.Scope",
			};

			try {
				const snapshot = await statusClient.getStatus(scope);
				if (!aborted) {
					setStatus(snapshot.rules, snapshot.vars);
				}
			} catch (err) {
				console.error("Failed to fetch status:", err);
				if (!aborted) clearStatus();
			}
		};

		fetchStatus();
		return () => {
			aborted = true;
		};
	}, [selectedId, setStatus, clearStatus]);
};
