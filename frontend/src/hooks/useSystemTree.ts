import { useEffect } from "react";
import { controlPlaneClient } from "../api/live/transport";
import { useStore } from "../store";
import type { SidebarNode } from "../store/types";
import { Scope, Scope_Kind } from "../api/gen/web_pb";

export const useSystemTree = () => {
	const setTree = useStore((s) => s.setSidebarTree);

	useEffect(() => {
		let aborted = false;

		(async () => {
			const scope: Scope = {
				kind: Scope_Kind.CONTROL_PLANE,
				id: "cp-1",
				$typeName: "web.Scope",
			};

			for await (const tree of controlPlaneClient.subscribeTree(scope)) {
				if (aborted) break;

				// Build node pools -> nodes
				const poolNodes: SidebarNode[] = tree.nodePools.map((pool) => ({
					id: pool.id,
					name: pool.name,
					isDir: true,
					kind: "node-pool",
					children: pool.nodes.map((n) => ({
						id: n.id,
						name: n.name,
						kind: "node",
						status: "connected", // TODO: replace with actual status
					})),
				}));

				// Wrap everything in the root control-plane node
				const sidebarTree: SidebarNode[] = [
					{
						id: tree.controlPlaneId,
						name: "Control Plane",
						isDir: true,
						kind: "control-plane",
						children: poolNodes,
					},
				];

				setTree(sidebarTree);
			}
		})();

		return () => {
			aborted = true;
		};
	}, [setTree]);
};
