import * as React from "react";
import { usePresence } from "../../hooks/usePresence";
import { useStore } from "../../store";
import { Scope } from "../../api/gen/web_pb";

const PresencePanel: React.FC = () => {
    // Start streaming presence updates
	usePresence();

	const users = useStore((s) => s.presence.users);
	const currentScope = useStore((s) => s.sidebar.selectedId);

	return (
		<div className="flex h-full flex-col border-l border-white/10 bg-neutral-900">
			<div className="border-b border-white/10 px-3 py-2 text-sm font-medium text-neutral-400">
				Presence
			</div>
			<div className="flex-1 space-y-1 overflow-auto p-2">
				{users.length === 0 ? (
					<div className="px-2 py-1 text-sm text-neutral-500">
						No users online
					</div>
				) : (
					users.map((u) => {
						const sameScope =
							u.scope && u.scope.id === currentScope;
						return (
							<div
								key={u.userId}
								className={`flex items-center gap-2 rounded-md px-2 py-1 text-sm ${
									sameScope
										? "bg-emerald-600/20 text-emerald-300"
										: "text-neutral-200 hover:bg-white/5"
								}`}
							>
								<span
									className={`inline-block h-2 w-2 rounded-full ${
										sameScope
											? "bg-emerald-400"
											: "bg-neutral-500"
									}`}
								/>
								<span className="truncate">{u.userId}</span>
							</div>
						);
					})
				)}
			</div>
		</div>
	);
};

export default PresencePanel;
