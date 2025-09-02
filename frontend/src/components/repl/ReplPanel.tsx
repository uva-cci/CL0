import { Button, TextField } from "@mui/material";
import { useStore } from "../../store";
import { PaneHeader } from "../common/PaneHeader";
import { useState } from "react";
import { useRepl } from "../../hooks/useRepl";

const ReplPanel = () => {
	const { send, clearHistory } = useRepl();
	const history = useStore((s) => s.repl.history);
	const session = useStore((s) => s.session);
	const userId = session?.userId;
	const sessionId = session?.sessionId;

	const [input, setInput] = useState("");

	const onSend = async () => {
		if (!input.trim()) return;
		await send(input);
		setInput("");
	};

	if (!userId || !sessionId) {
		return (
			<div className="flex h-full items-center justify-center text-neutral-500">
				No session/user ID
			</div>
		);
	}

	return (
		<div className="flex h-full flex-col">
			<PaneHeader
				title="REPL"
				subtitle={`Session: ${sessionId} · User: ${userId}`}
				right={
					<Button
						size="small"
						variant="outlined"
						onClick={clearHistory}
					>
						Clear
					</Button>
				}
			/>

			<div className="flex-1 space-y-2 overflow-auto p-3">
				{history.map((h, i) => (
					<div
						key={`${h.kind}-${h.id ?? i}`}
						className={`rounded p-2 text-sm ${
							h.kind === "notice"
								? "bg-yellow-500/10 text-yellow-200"
								: h.kind === "ack"
									? "bg-emerald-500/10 text-emerald-200"
									: h.kind === "output"
										? "bg-indigo-500/10 text-indigo-200"
										: "bg-white/5 text-neutral-200"
						}`}
					>
						<div className="whitespace-pre-wrap">{h.text}</div>
					</div>
				))}
			</div>

			<div className="flex gap-2 border-t border-white/10 p-3">
				<TextField
					fullWidth
					size="small"
					placeholder="Type a command… (Ctrl/Cmd+Enter to send)"
					value={input}
					onChange={(e) => setInput(e.target.value)}
					multiline
					minRows={1}
					onKeyDown={(e) => {
						if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
							e.preventDefault();
							onSend();
						}
					}}
				/>
				<Button variant="contained" onClick={onSend}>
					Send
				</Button>
			</div>
		</div>
	);
};

export default ReplPanel;
