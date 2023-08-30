<script script lang="ts">
	import Plot, { type Config, type Data, type PlotlyHTMLElement } from 'svelte-plotly.js';
	import type { PageServerData } from '$lib/$types';
	import { createLayout } from './graph';
	export let data: PageServerData;
	let inputId: string;
	let plot: PlotlyHTMLElement;

	let dpaData: Data[] = [
		{
			x: data.dpaData.x,
			y: data.dpaData.y,
			type: 'scatter',
			mode: 'markers',
			name: 'S12 Invite',
			marker: {
				color: data.dpaData.y,
				colorscale: 'Portland'
			},
			hoverinfo: 'x+y+text',
			hovertext: data.dpaData.labels
		}
	];

	const dpaLayout = createLayout(
		'Mean Bomb Damage vs Number of Attempts',
		'Number of Attempts',
		'Mean Bomb Damage'
	);

	const feedLayout = createLayout('Damage Ratio by Players', 'Player', 'Damage Ratio');

	const config: Partial<Config> = {
		responsive: true,
		watermark: false
	};

	async function handleSubmit() {
		if (inputId === null) {
			return;
		}

		if (/[a-zA-Z]/i.test(inputId)) {
			throw 'Only numbers allowed in input';
		}

		const req: RequestResponse = await (await fetch(`/api/bomb/${inputId}`)).json();

		let dpa_x: number[] = [];
		let attempt_y: number[] = [];
		let player_labels: string[] = [];

		const userTrace: Data = {
			x: dpa_x,
			y: attempt_y,
			type: 'scatter',
			mode: 'markers',
			name: 'Your Demo',
			marker: {
				symbol: 'star-dot',
				size: 14
			},
			hoverinfo: 'x+y+text',
			hovertext: player_labels
		};

		req.players.forEach((p: Player) => {
			dpa_x.push(p.damage_per_attempt);
			attempt_y.push(p.damage_per_attempt);
			player_labels.push(p.name);
		});

		if (dpaData.length > 1) {
			dpaData[1] = userTrace;
		} else {
			dpaData.push(userTrace);
		}

		// FIXME: Evil hack to force-update the plot
		const Plotly = (await import('plotly.js-dist')).default;
		Plotly.react(plot, dpaData, dpaLayout, [1], config);
	}
</script>

<title>Jump Efficiency</title>
<div class="container p-5 flex justify-center">
	<article
		class="prose lg:prose-xl prose-figure:max-w-lg prose-figure:rounded-xl prose-figure:shallow-md prose-figure:border"
	>
		<h1>Separating the Players from the Pretenders</h1>
		<p>
			Some soldiers have the reputation as "feeders" while others are respected for their knowledge
			of when to value their life. This quality is assigned by the "eye test" watching demos or
			looking at DPM and deaths in logs. I sought out to create a better metric which tracked
			individual jump attempts rather than running totals.
		</p>

		<h3>Introducing Jump Efficiency</h3>

		<p>
			Jump Efficiency is a stat created by measuring the average damage and kills per rocket jump
			attempt. Each "attempt" is started by a <code>RocketJumpStarted</code> event and ends at death
			or 3 seconds after the <code>RocketJumpLanded</code> event is triggered.
		</p>

		<p>
			Plotting the average bomb damage vs the number of attempts, we can see who is getting the most
			damage per the amount of total bomb attempts in the season.
		</p>

		<div class="flex justify-center">
			<figure class="shallow-md shadow-xl">
				<Plot data={dpaData} layout={dpaLayout} {config} debounce={250} bind:plot />
			</figure>
		</div>

		<p class="flex justify-center">Try it out with your own demo!</p>
		<form class="flex flex-row gap-2 justify-center" on:submit|preventDefault={handleSubmit}>
			<input
				type="text"
				placeholder="demos.tf id"
				class="rounded-md transition hover:shadow-md hover:ring hover:ring-sky-200"
				bind:value={inputId}
			/>
			<button
				class="
                flex
                items-center
                justify-center
                shadow-md
                rounded-md
                box-border
                h-10
                w-32
                bg-gradient-to-r
                from-indigo-500
                from-10%
                via-sky-500
                via-30%
                to-emerald-500
                to-90%
                transition
                hover:ring
                hover:ring-sky-200
                "
			>
				<p class="font-bold text-sky-50">Analyze!</p>
			</button>
		</form>

		<h3>The Biggest Feeders</h3>
		<p>
			Going a step further from before, we can quantify the "feed factor" by taking the ratio
			between damage recieved and damage dealt per bomb. The higher the ratio is, so is the "feed
			factor"
		</p>

		<p>
			...and the results seem to pass the eye-test as well. From the dataset of RGL Season 12 Invite
			scrims/matches including against Advanced players, <b>{data.feedData.labels[0]}</b> has the
			best damage ratio (lowest feed factor) of all players at
			<b>{data.feedData.y[0].toFixed(3)}</b> dt/dmg.
		</p>

		<div class="flex justify-center">
			<figure class="shallow-md shadow-xl">
				<Plot
					data={[
						{
							labels: data.feedData.labels,
							marker: {
								color: data.feedData.y,
								colorscale: 'Portland'
							},
							hoverinfo: 'y+text',
							hovertext: data.feedData.labels,
							y: data.feedData.y,
							type: 'bar',
							name: 'S12 Invite'
						}
					]}
					layout={feedLayout}
					{config}
					debounce={250}
				/>
			</figure>
		</div>

		<h3>Final Thoughts</h3>
		<p>
			Talk about creating the final statistic, picking the treshold to use and if bomb attempts for
			soldiers predicts or is correlated to the win percentage of the log
		</p>
		<p>
			A limitation of these statistics are the dataset itself. The dataset was created by scanning
			from logs.tf and matching a demo with the same players, map, and time. This may under
			represent players which scrim in servers without demos.tf auto upload enabled.
		</p>
	</article>
</div>
