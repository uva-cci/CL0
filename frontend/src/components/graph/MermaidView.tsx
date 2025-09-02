import * as React from "react";
import mermaid from "mermaid";
import { PaneHeader } from "../common/PaneHeader";

mermaid.initialize({ startOnLoad: false, theme: "dark" });

const MermaidView = () => {
	const [svg, setSvg] = React.useState<string>("");

	React.useEffect(() => {
		//const def = `graph TD\n  A[Start] --> B{Choice}\n  B -->|Yes| C[Approved]\n  B -->|No| D[Denied]`;
        const def = "graph TD\n";
		let cancelled = false;
		const render = async () => {
			const { svg } = await mermaid.render(`m-${Date.now()}`, def);
			if (!cancelled) setSvg(svg);
		};
		render();
		return () => {
			cancelled = true;
		};
	}, []);

	return (
		<div className="flex h-full flex-col">
			<PaneHeader title="Graph" subtitle="Mermaid.js" />
			<div className="flex-1 overflow-auto">
				<div className="min-h-full w-full grid place-items-center p-4">
					{/* Use a wrapper so we can center the injected SVG */}
					<div className="max-w-full" dangerouslySetInnerHTML={{ __html: svg }} />
				</div>
			</div>
		</div>
	);
};

export default MermaidView;
