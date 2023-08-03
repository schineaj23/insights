<script lang="ts">
	import Plot, { type Data, type PlotHoverEvent } from 'svelte-plotly.js';
	import type { PageServerData } from './$types';

	export let data: PageServerData;

	const plotData: Data[] = [
		{
			x: data.x,
			y: data.y,
			type: 'scatter',
			mode: 'markers',
			marker: {
				color: data.y,
				colorscale: 'Portland'
			},
			hoverinfo: 'x+y+text',
			hovertext: data.labels
		}
	];
</script>

<h1>Welcome to SvelteKit</h1>
<p>Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation</p>
<Plot
	data={plotData}
	layout={{
		title: 'Average Bomb Damage vs Number of Attempts',
		xaxis: { title: 'Average Bomb Damage' },
		yaxis: { title: 'Number of Attempts' },
		font: { family: 'Arial' },
		showlegend: false
	}}
	debounce={250}
/>
