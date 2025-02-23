import { goto } from '$app/navigation';

export async function load({ fetch }) {
    const feedResponse = await fetch("/api/posts/feed", {
        headers: {
            "Authorization": `Bearer ${window.localStorage.getItem("token")}`
        }
    });
    return {
        feed: await feedResponse.json(),
    };
}