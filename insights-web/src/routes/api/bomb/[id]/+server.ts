import { ANALYZE_FUNCTION_URL } from '$env/static/private';

export async function GET({ params }): Promise<Response> {
	return fetch(`${ANALYZE_FUNCTION_URL}?id=${params.id}`);
}
