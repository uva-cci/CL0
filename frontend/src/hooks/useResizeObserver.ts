import { useState, useEffect, useRef } from "react";

const useResizeObserver = <T extends HTMLElement>() => {
	const ref = useRef<T | null>(null);
	const [rect, setRect] = useState<DOMRect | null>(null);

	useEffect(() => {
		if (!ref.current) return;
		const el = ref.current;
		const obs = new ResizeObserver(() =>
			setRect(el.getBoundingClientRect()),
		);
		obs.observe(el);
		setRect(el.getBoundingClientRect());
		return () => obs.disconnect();
	}, []);

	return { ref, rect };
};

export { useResizeObserver };
