export async function GET({ params }) {
	return fetch(`http://127.0.0.1:9000?id=${params.id}`);
}
