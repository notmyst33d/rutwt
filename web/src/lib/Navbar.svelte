<script>
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
</script>

<div class="row padding-16 align-items-center wrap">
    <div class="row grow align-items-center">
        <!-- svelte-ignore a11y_missing_attribute -->
        <img src="/favicon.png" class="logo" />
        <a
            href="/"
            class="margin-left-8 {page.url.pathname == '/' ? 'bold' : ''}"
            >Лента</a
        >
        <a
            href="/latest"
            class="margin-left-8 {page.url.pathname == '/latest' ? 'bold' : ''}"
            >Последние посты</a
        >
    </div>
    <div class="row align-items-center">
        {#if page.url.pathname != "/post"}
            <button class="button" onclick={() => goto("/post")}
                >Сделать пост</button
            >
        {/if}
        <!-- svelte-ignore a11y_missing_attribute -->
        <!-- svelte-ignore a11y_consider_explicit_label -->
        <a href="/{page.data.user.username}">
            <img
                id="profile-picture"
                src={page.data.user.profile_picture_photo_id !== null
                    ? `/api/media/${page.data.user.profile_picture_photo_id}.jpg:small`
                    : "/placeholder/nopfp.png"}
                class="profile-picture margin-left-8"
            />
        </a>
    </div>
</div>
