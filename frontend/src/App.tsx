import { useStore } from "./store";
import LeftSidebar from "./components/layout/LeftSidebar";
import CenterPane from "./components/layout/CenterPane";
import RightPane from "./components/layout/RightPane";
import SessionGate from "./components/sessionGate/SessionGate";

const App = () => {
	const leftCompressed = useStore((s) => s.layout.leftCompressed);
	const { userId, sessionId } = useStore((s) => s.session);

	if (!userId || !sessionId) {
		// Not yet joined: show gate
		return <SessionGate />;
	}

	// Main app layout
	return (
		<div className="flex h-dvh w-dvw bg-neutral-950 text-neutral-100">
			<div
				className={
					leftCompressed
						? "w-[60px] border-r border-white/10"
						: "w-[280px] border-r border-white/10"
				}
			>
				<LeftSidebar />
			</div>

			<div className="grid min-h-0 min-w-0 flex-1 grid-cols-[1fr_420px]">
				<CenterPane />
				<RightPane />
			</div>
		</div>
	);
};

export default App;
