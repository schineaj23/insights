import prisma from '$lib/prisma';
import pl from 'nodejs-polars';
import type { PageServerLoad } from '$lib/$types';

export const load = (async ({ params }) => {
	const names = await prisma.player.findMany();
	const namesMap = new Map(names.map((i) => [i.steamid64, i.name]));

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

	let feedTabulation = grouped
		.agg(pl.col('damage').sum(), pl.col('damage_taken').sum())
		.select('player_id', pl.col('damage_taken').div(pl.col('damage')))
		.join(grouped.count(), { on: 'player_id' })
		.sort('damage_taken')
		.filter(pl.col('id_count').greaterThan(50));
	// Minimum 100 attempts, limit 50 so I can render all of them

	const feedLabels = feedTabulation
		.getColumn('player_id')
		.toArray()
		.map((id) => namesMap.get(id));
	const feedRatio = feedTabulation.getColumn('damage_taken').toArray();

	const playerIds = tabulation
		.getColumn('player_id')
		.toArray()
		.map((id) => namesMap.get(id));
	const attemptsByPlayer = tabulation.getColumn('id_count').toArray();
	const averageDamageByPlayer = tabulation.getColumn('damage').toArray();

	return {
		dpaData: {
			labels: playerIds,
			y: averageDamageByPlayer,
			x: attemptsByPlayer
		},
		feedData: {
			labels: feedLabels,
			y: feedRatio
		}
	};
}) satisfies PageServerLoad;
