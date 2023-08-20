import type * as Kit from '@sveltejs/kit';

type RouteParams = {};

export type PageServerLoad = Kit.ServerLoad<RouteParams>;
export type PageLoad = Kit.Load<RouteParams>;

export type PageServerData = {
	dpaData: DeathPerAttemptsData;
	feedData: FeedData;
};

export type DeathPerAttemptsData = {
	x: number[];
	y: number[];
	labels: string[];
};

export type FeedData = {
	y: number[];
	labels: string[];
};
