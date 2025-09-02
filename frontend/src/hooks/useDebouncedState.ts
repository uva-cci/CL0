import { useEffect, useState } from "react";

const useDebouncedState = <T>(initial: T, delay = 300) => {
	const [value, setValue] = useState<T>(initial);
	const [debounced, setDebounced] = useState<T>(initial);

	useEffect(() => {
		const t = setTimeout(() => setDebounced(value), delay);
		return () => clearTimeout(t);
	}, [value, delay]);

	return { value, setValue, debounced };
};
export { useDebouncedState };
