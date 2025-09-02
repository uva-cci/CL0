import { useState } from "react";
import {
	Button,
	TextField,
	Card,
	CardContent,
	Typography,
} from "@mui/material";
import { useStore } from "../../store";

const SessionGate = () => {
	const setSession = useStore((s) => s.setSession);
	const [userId, setUserId] = useState("");
	const [sessionId, setSessionId] = useState("");

	const submit = () => {
		if (!userId.trim() || !sessionId.trim()) return;
		setSession(userId.trim(), sessionId.trim());
	};

	return (
		<div className="flex h-dvh w-dvw items-center justify-center bg-neutral-950 text-neutral-100">
			<Card className="w-[360px] rounded-2xl bg-neutral-900 shadow-lg">
				<CardContent className="flex flex-col gap-4">
					<Typography variant="h6" className="text-center">
						Join Session
					</Typography>
					<TextField
						fullWidth
						label="User ID"
						size="small"
						value={userId}
						onChange={(e) => setUserId(e.target.value)}
						autoFocus
					/>
					<TextField
						fullWidth
						label="Session ID"
						size="small"
						value={sessionId}
						onChange={(e) => setSessionId(e.target.value)}
					/>
					<Button fullWidth variant="contained" onClick={submit}>
						Continue
					</Button>
				</CardContent>
			</Card>
		</div>
	);
};

export default SessionGate;