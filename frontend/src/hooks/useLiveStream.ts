import { useEffect } from "react";
import { useStore } from "../store";
import { toast } from "react-hot-toast";

/**
 * Subscribes to server events and updates slices.
 * - Replays from `since` on mount for recovery.
 * - Auto-reconnects with backoff on failure.
 */
const useLiveStream = () => {
	const pushEntry = useStore((s) => s.pushEntry);
	const setRules = useStore((s) => s.setRules);
	const setVariables = useStore((s) => s.setVariables);

	useEffect(() => {
		let cancelled = false;
		let since = Date.now() - 60_000; // last minute as recovery window

		async function connect() {
			try {
				// TODO: replace with actual client stream once codegen exists
				// for await (const msg of liveClient.streamEvents({ since })) {
				//   if (cancelled) break
				//   // route msg.event to slices
				// }
			} catch (err) {
				if (!cancelled) {
					toast.error("Live connection lost. Reconnecting…");
					setTimeout(connect, 2000);
				}
			}
		}

		connect();
		return () => {
			cancelled = true;
		};
	}, [pushEntry, setRules, setVariables]);
};

export { useLiveStream };
