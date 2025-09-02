import * as React from "react";
import { IconButton, Tooltip } from "@mui/material";
import ChevronRightIcon from "@mui/icons-material/ChevronRight";
import ChevronLeftIcon from "@mui/icons-material/ChevronLeft";
import SidebarTree from "../sidebar/SidebarTree";
import { useStore } from "../../store";
import PresencePanel from "../sidebar/PresencePanel";

const LeftSidebar = () => {
	const compressed = useStore((s) => s.layout.leftCompressed);
	const setCompressed = useStore((s) => s.setLeftCompressed);

	return (
		<div className="flex h-full flex-col">
			<div className="flex items-center justify-between p-2">
				<Tooltip title={compressed ? "Expand" : "Collapse"}>
					<IconButton
						size="small"
						onClick={() => setCompressed(!compressed)}
					>
						{compressed ? (
							<ChevronRightIcon />
						) : (
							<ChevronLeftIcon />
						)}
					</IconButton>
				</Tooltip>
			</div>
			<div className="flex-1 overflow-hidden">
				<SidebarTree compressed={compressed} />
			</div>
			{!compressed && (
				<div className="h-48 border-t border-white/10">
					<PresencePanel />
				</div>
			)}
		</div>
	);
};

export default LeftSidebar;
