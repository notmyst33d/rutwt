<script>
    import { page } from "$app/state";
    import Navbar from "$lib/Navbar.svelte";
    import Post from "$lib/Post.svelte";
</script>

<div class="column">
    <Navbar />
    <span class="action align-self-center">Последние посты</span>
    <div id="posts-container" class="column feed-container">
        {#each page.data.latestPosts as post}
            <Post {post} />
        {/each}
    </div>
</div>
