<script>
    import { invalidate } from "$app/navigation";
    import { page } from "$app/state";
    import Navbar from "$lib/Navbar.svelte";
    import Post from "$lib/Post.svelte";
</script>

<div class="column">
    <Navbar />
    {#if page.data.userPage === undefined}
        <span class="action align-self-center">Пользователь не найден</span>
    {:else}
        <div class="column feed-container">
            <div class="column profile margin-bottom-8">
                {#if page.data.userPage.banner_photo_id !== null}
                    <!-- svelte-ignore a11y_missing_attribute -->
                    <img
                        class="header"
                        src="/api/media/{page.data.userPage
                            .banner_photo_id}.jpg"
                    />
                {:else}
                    <div class="header gray-bg"></div>
                {/if}
                <!-- svelte-ignore a11y_missing_attribute -->
                <div class="row">
                    <img
                        class="big-profile-picture margin-top-8"
                        src={page.data.userPage.profile_picture_photo_id !==
                        null
                            ? `/api/media/${page.data.userPage.profile_picture_photo_id}.jpg`
                            : "/placeholder/nopfp.png"}
                    />
                    <div class="column margin-left-8 margin-top-8">
                        <div class="row wrap gap align-items-center">
                            <span class="big-realname"
                                >{page.data.userPage.realname}</span
                            >
                            {#if page.data.userPage.followers === 1}
                                <span class="subs hack">1 подписчик</span>
                            {:else}
                                <span class="subs hack"
                                    >{page.data.userPage.followers} подписчиков</span
                                >
                            {/if}
                        </div>
                        <span class="big-username"
                            >@{page.data.userPage.username}</span
                        >
                        <div class="text margin-top-8">
                            {page.data.userPage.bio}
                        </div>
                    </div>
                </div>
                {#if page.data.userPage.id != page.data.user.id}
                    <!-- svelte-ignore a11y_consider_explicit_label -->
                    <button
                        class="button modal-button margin-top-8 {page.data
                            .userPage.following
                            ? 'liked-button'
                            : ''}"
                        onclick={async () => {
                            if (page.data.userPage.following) {
                                await fetch(
                                    `/api/users/unfollow?id=${page.data.userPage.id}`,
                                    {
                                        headers: {
                                            Authorization: `Bearer ${window.localStorage.getItem("token")}`,
                                        },
                                    },
                                );
                                page.data.userPage.following = false;
                                page.data.userPage.followers -= 1;
                            } else {
                                await fetch(
                                    `/api/users/follow?id=${page.data.userPage.id}`,
                                    {
                                        headers: {
                                            Authorization: `Bearer ${window.localStorage.getItem("token")}`,
                                        },
                                    },
                                );
                                page.data.userPage.following = true;
                                page.data.userPage.followers += 1;
                            }
                        }}
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            height="24px"
                            viewBox="0 -960 960 960"
                            width="24px"
                            fill="currentColor"
                            ><path
                                d="M160-200v-80h80v-280q0-83 50-147.5T420-792v-28q0-25 17.5-42.5T480-880q25 0 42.5 17.5T540-820v28q80 20 130 84.5T720-560v280h80v80H160Zm320-300Zm0 420q-33 0-56.5-23.5T400-160h160q0 33-23.5 56.5T480-80ZM320-280h320v-280q0-66-47-113t-113-47q-66 0-113 47t-47 113v280Z"
                            /></svg
                        >
                    </button>
                {:else}
                    <!-- svelte-ignore a11y_consider_explicit_label -->
                    <a href="/settings" class="fit-content">
                        <button class="button modal-button margin-top-8">
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                height="24px"
                                viewBox="0 -960 960 960"
                                width="24px"
                                fill="currentColor"
                                ><path
                                    d="M200-200h57l391-391-57-57-391 391v57Zm-40 80q-17 0-28.5-11.5T120-160v-97q0-16 6-30.5t17-25.5l505-504q12-11 26.5-17t30.5-6q16 0 31 6t26 18l55 56q12 11 17.5 26t5.5 30q0 16-5.5 30.5T817-647L313-143q-11 11-25.5 17t-30.5 6h-97Zm600-584-56-56 56 56Zm-141 85-28-29 57 57-29-28Z"
                                /></svg
                            >
                        </button>
                    </a>
                {/if}
            </div>
            {#if page.data.user.username == page.params.slug && (page.data.userPosts === undefined || page.data.userPosts.length === 0)}
                <span class="action align-self-center"
                    >У вас пока что нет постов</span
                >
            {:else if page.data.userPosts === undefined || page.data.userPosts.length === 0}
                <span class="action align-self-center"
                    >У данного пользователя пока что нет постов</span
                >
            {:else}
                {#each page.data.userPosts as post}
                    <Post {post} />
                {/each}
            {/if}
        </div>
    {/if}
</div>

<style>
    .subs {
        color: #686868;
    }

    .gap {
        column-gap: 8px;
    }

    .hack {
        height: calc(100% - 16px);
    }
</style>
