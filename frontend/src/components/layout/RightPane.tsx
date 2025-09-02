import PolicyEditor from "../policy/PolicyEditor";
import RulesPanel from "../policy/RulesPanel";
import VariablesPanel from "../policy/VariablesPanel";
import { useStore } from "../../store";

const RightPane = () => {
	return (
		<div className="flex flex-col h-full">
            <div className="h-4/12">
			    <PolicyEditor />
            </div>
            
			<div className="h-8/12">
				<RulesPanel />
				<VariablesPanel />
            </div>
		</div>
	);
};
export default RightPane;
