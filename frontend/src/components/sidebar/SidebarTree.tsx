import * as React from "react";
import { Tree, NodeApi, type NodeRendererProps } from "react-arborist";
import FolderIcon from "@mui/icons-material/Folder";
import FolderOpenIcon from "@mui/icons-material/FolderOpen";
import HideSourceIcon from "@mui/icons-material/HideSource";
import PanoramaFishEyeIcon from "@mui/icons-material/PanoramaFishEye";
import { useStore } from "../../store";
import { useResizeObserver } from "../../hooks/useResizeObserver";
import type { SidebarNode } from "../../store/types";
import { useSystemTree } from "../../hooks/useSystemTree";

type RowProps = {
	node: NodeApi<SidebarNode>;
	innerRef: (el: HTMLDivElement | null) => void;
	attrs: React.HTMLAttributes<HTMLDivElement> & { className?: string };
	children: React.ReactElement;
};

const Row: React.FC<RowProps> = ({ node, innerRef, attrs, children }) => {
	const { className: attrClass, ...rest } = attrs;
	const cls = [
		attrClass ?? "",
		"group rounded-md px-1",
		node.isSelected
			? "bg-white/10 ring-1 ring-white/10"
			: "hover:bg-white/5",
	]
		.filter(Boolean)
		.join(" ");
	return (
		<div ref={innerRef} {...rest} className={cls}>
			{children}
		</div>
	);
};

const NodeRow: React.FC<
	NodeRendererProps<SidebarNode> & { compressed: boolean }
> = ({ node, compressed }) => {
	const isFolder = !!(
		node.data?.isDir ||
		(!node.isLeaf && node.children?.length)
	);
	const fileStatus = node.data?.status;
	return (
		<div
			className={`flex items-center gap-2 ${compressed ? "px-3" : "px-2"} cursor-pointer py-1 text-sm text-neutral-200`}
			onDoubleClick={() => isFolder && node.toggle()}
		>
			{isFolder ? (
				<button
					type="button"
					onClick={(e) => {
						e.stopPropagation();
						node.toggle();
					}}
					className="grid place-items-center"
					aria-label={node.isOpen ? "Collapse" : "Expand"}
				>
					{node.isOpen ? (
						<FolderOpenIcon
							fontSize="medium"
							className="cursor-pointer text-white"
						/>
					) : (
						<FolderIcon
							fontSize="medium"
							className="cursor-pointer text-white"
						/>
					)}
				</button>
			) : (
				<span className={compressed ? "pl-0" : "pl-5"}>
					{fileStatus === "disconnected" ? (
						<HideSourceIcon
							fontSize="medium"
							className="text-neutral-400"
						/>
					) : (
						<PanoramaFishEyeIcon
							fontSize="medium"
							className="text-emerald-400"
						/>
					)}
				</span>
			)}
			<span className="truncate">
				{node.data?.name ?? String(node.id)}
			</span>
		</div>
	);
};

const SidebarTree = ({ compressed }: { compressed: boolean }) => {
	const tree = useStore((s) => s.sidebar.tree);
	const setSelected = useStore((s) => s.setSidebarSelected);
	const { ref, rect } = useResizeObserver<HTMLDivElement>();

	// hook to subscribe to backend tree
	useSystemTree();

	return (
		<div
			ref={ref}
			className={compressed ? "h-full w-full px-1" : "h-full w-full px-2"}
		>
			<Tree<SidebarNode>
				data={tree}
				openByDefault
				selectionFollowsFocus
				rowHeight={28}
				width={rect?.width ?? 0}
				height={rect?.height ?? 0}
				renderRow={Row}
				onSelect={(nodes) => setSelected(nodes[0]?.id)}
			>
				{(props) => <NodeRow {...props} compressed={compressed} />}
			</Tree>
		</div>
	);
};

export default SidebarTree;