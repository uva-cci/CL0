import { DataGrid, GridColDef } from "@mui/x-data-grid";
import { useStore } from "../../store";
import { PaneHeader } from "../common/PaneHeader";
import { useStatus } from "../../hooks/useStatus";

const columns: GridColDef[] = [
	{ field: "namespace", headerName: "Namespace", flex: 1 },
	{ field: "name", headerName: "Definition", flex: 2 },
	{ field: "enabled", headerName: "Enabled", type: "boolean", width: 110 },
];

const RulesPanel = () => {
	useStatus();
	const rows = useStore((s) => s.status.rules);

	return (
		<div className="flex flex-col">
			<PaneHeader title="Rules" subtitle="Live status" />
			<div className="flex-1">
				<DataGrid
					rows={rows}
					columns={columns}
					getRowId={(r) => r.id}
					density="compact"
					disableColumnMenu
				/>
			</div>
		</div>
	);
};
export default RulesPanel;
