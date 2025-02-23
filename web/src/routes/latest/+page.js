import { goto } from '$app/navigation';

export async function load({ fetch }) {
    const latestPostsResponse = await fetch("/api/posts/find", {
        headers: {
            "Authorization": `Bearer ${window.localStorage.getItem("token")}`
        }
    });
    return {
        latestPosts: await latestPostsResponse.json(),
    };
}