import { goto } from '$app/navigation';

export async function load({ params, fetch, depends }) {
    depends("data:comments");
    const userPostResponse = await fetch(`/api/posts/find?id=${params.id}&username=${params.username}`, {
        headers: {
            "Authorization": `Bearer ${window.localStorage.getItem("token")}`
        }
    });
    const commentsResponse = await fetch(`/api/posts/comments?id=${params.id}&username=${params.username}`, {
        headers: {
            "Authorization": `Bearer ${window.localStorage.getItem("token")}`
        }
    });
    return {
        userPost: userPostResponse.status === 200 ? await userPostResponse.json() : undefined,
        comments: commentsResponse.status === 200 ? await commentsResponse.json() : undefined,
    };
}