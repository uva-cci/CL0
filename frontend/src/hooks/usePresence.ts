import { useEffect } from "react";
import { presenceClient } from "../api/live/transport";
import { useStore } from "../store";
import { PresenceUpdate_Kind } from "../api/gen/web_pb";

export const usePresence = () => {
	const setPresence = useStore((s) => s.setPresence);
	const upsert = useStore((s) => s.upsertPresence);
	const remove = useStore((s) => s.removePresence);

	useEffect(() => {
		let aborted = false;

		(async () => {
			for await (const evt of presenceClient.subscribe({
				$typeName: "web.Empty",
			})) {
				if (aborted) break;

				const k = evt.kind;
				if (!k) continue;

				switch (k.case) {
					case "snapshot":
						setPresence(k.value.users);
						break;
					case "update":
						if (!k.value.user) break;
						switch (k.value.kind) {
							case PresenceUpdate_Kind.JOINED:
							case PresenceUpdate_Kind.MOVED:
								upsert(k.value.user);
								break;
							case PresenceUpdate_Kind.LEFT:
								remove(k.value.user.userId);
								break;
						}
						break;
				}
			}
		})();

		return () => {
			aborted = true;
		};
	}, [setPresence, upsert, remove]);
};
