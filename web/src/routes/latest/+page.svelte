<script>
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import InfiniteScroll from "$lib/InfiniteScroll.svelte";
    import Navbar from "$lib/Navbar.svelte";
    import Post from "$lib/Post.svelte";

    let posts = $state(page.data.latestPosts);
    $effect(() => {
        posts = page.data.latestPosts;
    });

    let noMoreData = $state(false);
</script>

<div class="column">
    <Navbar />
    <span class="action align-self-center">Последние посты</span>
    <div id="posts-container" class="column feed-container">
        {#each posts as _, i}
            <Post bind:post={posts[i]} />
        {/each}
        {#if noMoreData}
            <span class="action align-self-center">Вы долистали до конца</span>
        {/if}
    </div>
    <InfiniteScroll
        bind:noMoreData
        onLoad={(data) => (posts = posts.concat(data))}
    />
</div>
