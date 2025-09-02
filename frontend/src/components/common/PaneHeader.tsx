import * as React from "react";

export const PaneHeader = ({
	title,
	subtitle,
	right,
}: {
	title: string;
	subtitle?: string;
	right?: React.ReactNode;
}) => {
	return (
		<div className="flex items-center justify-between px-3 py-2">
			<div>
				<div className="text-xs tracking-wide text-white/50 uppercase">
					{subtitle}
				</div>
				<div className="text-lg font-semibold">{title}</div>
			</div>
			{right}
		</div>
	);
}
