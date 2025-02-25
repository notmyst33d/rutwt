<script>
    import { page } from "$app/state";
    import Navbar from "$lib/Navbar.svelte";
    import Post from "$lib/Post.svelte";
    import PostForm from "$lib/PostForm.svelte";

    let userPost = $derived.by(() => {
        let userPost = $state(page.data.userPost);
        return userPost;
    });
    let comments = $derived.by(() => {
        let comments = $state(page.data.comments);
        return comments;
    });
</script>

<div class="column">
    <Navbar />
    <div class="column feed-container">
        {#if userPost === undefined || userPost.length === 0}
            <span class="action align-self-center">Пост не найден</span>
        {:else}
            <Post bind:post={userPost[0]} />
            <span class="action align-self-center margin-top-8 margin-bottom-8"
                >Комментарии</span
            >
            <PostForm commentPostId={userPost[0].id} />
            <div class="margin-top-8"></div>
            {#each comments as _, i}
                <Post bind:post={comments[i]} />
            {/each}
        {/if}
    </div>
</div>
