export async function GET({ params }) {
	return fetch(`http://localhost:9000?id=${params.id}`);
}
