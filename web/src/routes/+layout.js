export const ssr = false;
export const prerender = false;

import { goto } from '$app/navigation';
import { page } from '$app/state';

export async function load({ untrack, url, params, fetch }) {
    if (untrack(() => url.pathname === "/login" || url.pathname === "/register")) {
        return { user: undefined };
    }
    if (untrack(() => window.localStorage.getItem("token") === null)) {
        goto("/login");
        return { user: undefined };
    }
    const userResponse = await fetch("/api/users", {
        headers: {
            "Authorization": `Bearer ${window.localStorage.getItem("token")}`
        }
    });
    if (untrack(() => userResponse.status !== 200)) {
        goto("/login");
        return { user: undefined };
    }
    return {
        user: await userResponse.json(),
    };
}