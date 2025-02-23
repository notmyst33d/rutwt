<script>
    import { page } from "$app/state";
    import Navbar from "$lib/Navbar.svelte";
    import Post from "$lib/Post.svelte";
    import PostForm from "$lib/PostForm.svelte";
</script>

<div class="column">
    <Navbar />
    <div class="column feed-container">
        {#if page.data.userPost === undefined || page.data.userPost.length === 0}
            <span class="action align-self-center">Пост не найден</span>
        {:else}
            <Post post={page.data.userPost[0]} />
            <span class="action align-self-center margin-top-8 margin-bottom-8"
                >Комментарии</span
            >
            <PostForm commentPostId={page.data.userPost[0].id} />
            <div class="margin-top-8"></div>
            {#each page.data.comments as post}
                <Post {post} />
            {/each}
        {/if}
    </div>
</div>
