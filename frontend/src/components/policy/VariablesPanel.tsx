import { DataGrid, GridColDef } from "@mui/x-data-grid";
import { useStore } from "../../store";
import { PaneHeader } from "../common/PaneHeader";
import { useStatus } from "../../hooks/useStatus";

const columns: GridColDef[] = [
	{ field: "name", headerName: "Variable", flex: 1 },
	{ field: "enabled", headerName: "Enabled", type: "boolean", width: 110 },
];

const VariablesPanel = () => {
	useStatus();
	const rows = useStore((s) => s.status.vars);

	return (
		<div className="flex flex-col">
			<PaneHeader title="Variables" subtitle="Live status" />
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
export default VariablesPanel;
