interface Player {
	name: string;
	steamid: number;
	attempts: number;
	damage_per_attempt: number;
}

interface RequestResponse {
	players: Player[];
}
