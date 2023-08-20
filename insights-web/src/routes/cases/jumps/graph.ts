import type { Layout } from 'svelte-plotly.js';

export function createLayout(title: string, xaxis: string, yaxis: string): Partial<Layout> {
	let plotLayout: Partial<Layout> = {
		title: title,
		xaxis: { title: xaxis },
		yaxis: { title: yaxis },
		font: {
			family:
				'ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans", sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji"'
		},
		autosize: true,
		showlegend: true,
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

	return plotLayout;
}
