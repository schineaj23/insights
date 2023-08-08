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

<div class="container max-w-xl my-10 p-10 shadow-md">
	<div class="flex flex-col justify-center gap-4">
		<p class="text-5xl font-bold subpixel-antialiased">Tracking Bomb Efficiency</p>
		<p class="">
			Lorem ipsum dolor sit amet consectetur adipisicing elit. Maiores ipsa dolorem voluptatem
			dignissimos eaque. Sunt sapiente facilis tempore, debitis doloremque quisquam aut voluptatem
			illo itaque tempora ipsum dicta, sequi porro?
		</p>

		<div class="max-w-lg rounded-xl shadow-md border">
			<Plot
				data={plotData}
				layout={{
					title: 'Average Bomb Damage vs Number of Attempts',
					xaxis: { title: 'Average Bomb Damage' },
					yaxis: { title: 'Number of Attempts' },
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
					}
				}}
				debounce={250}
			/>
		</div>

		<div class="flex flex-row gap-2">
			<input
				type="text"
				placeholder=" demos.tf id"
				class="rounded-md transition hover:shadow-md hover:ring hover:ring-sky-200"
			/>
			<div
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
			</div>
		</div>
	</div>
</div>
