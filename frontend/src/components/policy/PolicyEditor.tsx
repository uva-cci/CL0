import * as React from "react";
import Editor from "@monaco-editor/react";
import { useStore } from "../../store";
import { PaneHeader } from "../common/PaneHeader";

const PolicyEditor = () => {
	const json = useStore((s) => s.policy.json);
	const setJson = useStore((s) => s.setPolicy);

	return (
		<div className="flex flex-col h-full">
			<PaneHeader title="Policy" subtitle="JSON" />
			<div className="flex-1 h-3/12 relative">
				<div className="absolute inset-0">
					<Editor
						height="100%"
						defaultLanguage="json"
						value={json}
						theme={"vs-dark"}
						onChange={(v) => setJson(v ?? "")}
						options={{
							minimap: { enabled: false },
							fontSize: 14,
							automaticLayout: true,
						}}
					/>
				</div>
			</div>
		</div>
	);
};
export default PolicyEditor;
