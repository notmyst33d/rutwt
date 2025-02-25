<script>
    import { page } from "$app/state";
    import InfiniteScroll from "$lib/InfiniteScroll.svelte";
    import Navbar from "$lib/Navbar.svelte";
    import Post from "$lib/Post.svelte";

    let posts = $state(page.data.feed);
    $effect(() => {
        posts = page.data.feed;
    });

    let noMoreData = $state(false);
</script>

<div class="column">
    <Navbar />
    <span class="action align-self-center">Ваша лента</span>
    {#if posts === undefined || posts.length === 0}
        <div id="feed-empty" class="column">
            <div class="text-center align-self-center">
                Подпишитесь на пользователей Хъ чтобы ваша лента имела
                активность
            </div>
            <div class="text-center align-self-center">
                Проверьте вкладку <a href="/latest">Последние посты</a> для того
                чтобы найти для себя новый контент
            </div>
        </div>
    {:else}
        <div id="posts-container" class="column feed-container">
            {#each posts as _, i}
                <Post bind:post={posts[i]} />
            {/each}
            {#if noMoreData}
                <span class="action align-self-center"
                    >Вы долистали до конца</span
                >
            {/if}
        </div>
    {/if}
    <InfiniteScroll
        bind:noMoreData
        onLoad={(data) => (posts = posts.concat(data))}
        additionalQueryParams="&feed=true"
    />
</div>
