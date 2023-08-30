import prisma from '$lib/prisma';
import pl from 'nodejs-polars';
import type { PageServerLoad } from '$lib/$types';

export const load = (async ({ params }) => {
	const bigQuery = await prisma.$queryRaw`
    select
      player_id,
      player.name,
      num_logs,
      num_attempts,
      attempts_per_log,
      damage_per_attempt,
	  damage_ratio
    from
    (
      select
        player_id,
        count(distinct log_id) as "num_logs", count(log_id) as "num_attempts",
        count(log_id)/count(distinct log_id)::float as "attempts_per_log",
        sum(damage)/count(log_id)::float as "damage_per_attempt",
		    sum(damage_taken)/sum(damage)::float as "damage_ratio"
      from
        bomb_attempt
      group by
        player_id
    )
    as
      tab
    left join
      player
    on
      player.steamid64 = player_id
    where
      num_attempts > 50
    order by
      damage_ratio asc`;

	let df = pl.DataFrame(bigQuery);

	const playerNames = df.getColumn('name').toArray();
	const averageDamageByPlayer = df.getColumn('damage_per_attempt').toArray();
	const attemptsByPlayerPerLog = df.getColumn('attempts_per_log').toArray();
	const feedRatio = df.getColumn('damage_ratio').toArray();

	return {
		dpaData: {
			labels: playerNames,
			y: averageDamageByPlayer,
			x: attemptsByPlayerPerLog
		},
		feedData: {
			labels: playerNames,
			y: feedRatio
		}
	};
}) satisfies PageServerLoad;
