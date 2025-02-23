<script>
    export let post;
</script>

<div class="message">
    <div class="row">
        <!-- svelte-ignore a11y_missing_attribute -->
        <!-- svelte-ignore a11y_consider_explicit_label -->
        <a href="/{post.user.username}">
            <img
                id="profile-picture"
                src={post.user.profile_picture_photo_id !== null
                    ? `/api/media/${post.user.profile_picture_photo_id}.jpg:small`
                    : "/placeholder/nopfp.png"}
                class="profile-picture"
            />
        </a>
        <div class="column margin-left-8">
            <span class="realname">{post.user.realname}</span>
            <span class="username">@{post.user.username}</span>
        </div>
    </div>
    <span class="text">{post.message}</span>
    <div class="media">
        {#each post.media as media}
            {#if media.photo !== null}
                <!-- svelte-ignore a11y_missing_attribute -->
                <img loading="lazy" src="/api/media/{media.photo}.jpg" />
            {/if}
        {/each}
    </div>
    <div class="row margin-top-8">
        <!-- svelte-ignore a11y_consider_explicit_label -->
        <button
            class="row align-items-center button post-button {post.liked
                ? 'liked-button'
                : ''}"
            onclick={async () => {
                if (post.liked) {
                    const response = await fetch(
                        `/api/posts/unlike?id=${post.id}`,
                        {
                            headers: {
                                Authorization: `Bearer ${window.localStorage.getItem("token")}`,
                            },
                        },
                    );
                    post.like_count -= 1;
                    post.liked = false;
                } else {
                    let response = await fetch(
                        `/api/posts/like?id=${post.id}`,
                        {
                            headers: {
                                Authorization: `Bearer ${window.localStorage.getItem("token")}`,
                            },
                        },
                    );
                    post.like_count += 1;
                    post.liked = true;
                }
            }}
        >
            <span class="margin-right-4">{post.like_count}</span>
            <!-- svelte-ignore a11y_missing_attribute -->
            {#if post.liked}
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    height="24px"
                    viewBox="0 -960 960 960"
                    width="24px"
                    fill="currentColor"
                    ><path
                        d="M480-147q-14 0-28.5-5T426-168l-69-63q-106-97-191.5-192.5T80-634q0-94 63-157t157-63q53 0 100 22.5t80 61.5q33-39 80-61.5T660-854q94 0 157 63t63 157q0 115-85 211T602-230l-68 62q-11 11-25.5 16t-28.5 5Z"
                    /></svg
                >
            {:else}
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    height="24px"
                    viewBox="0 -960 960 960"
                    width="24px"
                    fill="currentColor"
                    ><path
                        d="M480-147q-14 0-28.5-5T426-168l-69-63q-106-97-191.5-192.5T80-634q0-94 63-157t157-63q53 0 100 22.5t80 61.5q33-39 80-61.5T660-854q94 0 157 63t63 157q0 115-85 211T602-230l-68 62q-11 11-25.5 16t-28.5 5Zm-38-543q-29-41-62-62.5T300-774q-60 0-100 40t-40 100q0 52 37 110.5T285.5-410q51.5 55 106 103t88.5 79q34-31 88.5-79t106-103Q726-465 763-523.5T800-634q0-60-40-100t-100-40q-47 0-80 21.5T518-690q-7 10-17 15t-21 5q-11 0-21-5t-17-15Zm38 189Z"
                    /></svg
                >
            {/if}
        </button>
        <!-- svelte-ignore a11y_consider_explicit_label -->
        <a href="/{post.user.username}/status/{post.id}" class="no-underline">
            <button
                class="row align-items-center button post-button margin-left-8"
            >
                <span class="margin-right-4">{post.comment_count}</span>
                <!-- svelte-ignore a11y_missing_attribute -->
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    height="24px"
                    viewBox="0 -960 960 960"
                    width="24px"
                    fill="currentColor"
                    ><path
                        d="M280-400h400q17 0 28.5-11.5T720-440q0-17-11.5-28.5T680-480H280q-17 0-28.5 11.5T240-440q0 17 11.5 28.5T280-400Zm0-120h400q17 0 28.5-11.5T720-560q0-17-11.5-28.5T680-600H280q-17 0-28.5 11.5T240-560q0 17 11.5 28.5T280-520Zm0-120h400q17 0 28.5-11.5T720-680q0-17-11.5-28.5T680-720H280q-17 0-28.5 11.5T240-680q0 17 11.5 28.5T280-640ZM160-240q-33 0-56.5-23.5T80-320v-480q0-33 23.5-56.5T160-880h640q33 0 56.5 23.5T880-800v623q0 27-24.5 37.5T812-148l-92-92H160Zm594-80 46 45v-525H160v480h594Zm-594 0v-480 480Z"
                    /></svg
                >
            </button>
        </a>
    </div>
</div>
