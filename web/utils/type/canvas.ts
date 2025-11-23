export type Station = {
	id: number;
	pos_x: number;
	pos_y: number;
	types: ("taxi" | "bus" | "underground")[];
};

export type Connection = {
	from: number;
	to: number;
	mode: "taxi" | "bus" | "underground" | "water";
};
