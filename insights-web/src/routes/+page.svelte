<script script lang="ts">
	import Plot, { type Data, type Layout, type PlotlyHTMLElement } from 'svelte-plotly.js';
	import type { PageServerData } from './$types';
	export let data: PageServerData;
	let inputId: string;
	let plot: PlotlyHTMLElement;

	let plotData: Data[] = [
		{
			x: data.x,
			y: data.y,
			type: 'scatter',
			mode: 'markers',
			name: 'S12 Invite',
			marker: {
				color: data.y,
				colorscale: 'Portland'
			},
			hoverinfo: 'x+y+text',
			hovertext: data.labels
		}
	];

	let plotLayout: Partial<Layout> = {
		title: 'Mean Bomb Damage vs Number of Attempts',
		xaxis: { title: 'Number of Attempts' },
		yaxis: { title: 'Mean Bomb Damage' },
		font: {
			family:
				'ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans", sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji"'
		},
		autosize: true,
		showlegend: false,
		paper_bgcolor: 'transparent',
		plot_bgcolor: 'transparent',
		modebar: {
			activecolor: '#082f49',
			color: '#082f49',
			bgcolor: 'transparent',
			orientation: 'v',
			remove: 'lasso2d'
		},
		legend: {
			orientation: 'h'
		}
	};

	interface Player {
		name: string;
		steamid: number;
		attempts: number;
		damage_per_attempt: number;
	}

	interface RequestResponse {
		players: Player[];
	}

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

		if (plotData.length > 1) {
			plotData[1] = userTrace;
		} else {
			plotData.push(userTrace);
		}

		// FIXME: Evil hack to force-update the plot
		const Plotly = (await import('plotly.js-dist')).default;
		Plotly.react(plot, plotData, { ...plotLayout, showlegend: true }, [1]);
	}
</script>

<div class="container max-w-xl my-10 p-10 shadow-md">
	<div class="flex flex-col justify-center gap-4">
		<p class="text-5xl font-bold subpixel-antialiased">Tracking Jump Efficiency</p>
		<p class="">
			Lorem ipsum dolor sit amet consectetur adipisicing elit. Maiores ipsa dolorem voluptatem
			dignissimos eaque. Sunt sapiente facilis tempore, debitis doloremque quisquam aut voluptatem
			illo itaque tempora ipsum dicta, sequi porro?
		</p>

		<div class="max-w-lg rounded-xl shadow-md border">
			<Plot
				data={plotData}
				layout={plotLayout}
				config={{ responsive: true }}
				debounce={250}
				bind:plot
			/>
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
		<p>helo</p>
	</div>
</div>
