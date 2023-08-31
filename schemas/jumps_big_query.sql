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
        trunc(count(log_id)/count(distinct log_id)::numeric, 2) as "attempts_per_log",
        trunc(sum(damage)/count(log_id)::numeric, 2) as "damage_per_attempt",
		trunc(sum(damage_taken)/sum(damage)::numeric, 3) as "damage_ratio"
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
	  damage_ratio asc;