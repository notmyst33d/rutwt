import { goto } from '$app/navigation';
import { writable } from 'svelte/store';

export async function load({ params, fetch, depends }) {
    depends("data:userPage");
    const userPageResponse = await fetch(`/api/users/${params.slug}`, {
        headers: {
            "Authorization": `Bearer ${window.localStorage.getItem("token")}`
        }
    });
    const userPostsResponse = await fetch(`/api/posts/find?username=${params.slug}`, {
        headers: {
            "Authorization": `Bearer ${window.localStorage.getItem("token")}`
        }
    });
    return {
        userPage: userPageResponse.status === 200 ? await userPageResponse.json() : undefined,
        userPosts: userPostsResponse.status === 200 ? await userPostsResponse.json() : undefined,
    };
}