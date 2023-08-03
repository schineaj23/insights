import prisma from '$lib/prisma';
import pl from 'nodejs-polars';
import type { PageServerLoad } from './$types';

export const load = (async ({ params }) => {
	// TODO: move this to lib files. This is a PoC/Mockup

	const data = await prisma.bomb_attempt.findMany();
	let df = pl.DataFrame(data);

	let grouped = df.groupBy('player_id');
	let tabulation = grouped
		.mean()
		.select('player_id', 'damage')
		.join(grouped.count(), { on: 'player_id' })
		.sort('player_id')
		.filter(pl.col('id_count').greaterThan(50));
	// Minimum 50 attempts

	const playerIds = tabulation.getColumn('player_id').toArray();
	const attemptsByPlayer = tabulation.getColumn('id_count').toArray();
	const averageDamageByPlayer = tabulation.getColumn('damage').toArray();

	return {
		labels: playerIds,
		x: averageDamageByPlayer,
		y: attemptsByPlayer
	};
}) satisfies PageServerLoad;
