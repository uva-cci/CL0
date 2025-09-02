import * as React from "react";
import MermaidView from "../graph/MermaidView";
import ReplPanel from "../repl/ReplPanel";
import { useStore } from "../../store";

const CenterPane = () => {

	return (
        <div className="flex flex-col h-full">
			<div className="min-h-0 h-7/12 border-b border-white/10">
				<MermaidView />
			</div>
			<div className="min-h-0 h-5/12">
				<ReplPanel />
			</div>
		</div>
	);
};
export default CenterPane;
